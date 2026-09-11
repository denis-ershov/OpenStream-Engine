use ipnet::IpNet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Запись в таблице маршрутизации IP
#[derive(Debug, Clone)]
struct RouteEntry<T> {
    net: IpNet,
    value: T,
}

/// Таблица сопоставления IP-адресов по принципу Longest Prefix Match (LPM).
///
/// Порядок сортировки поддерживается лениво (`finalize`), а не при каждой
/// вставке: пересортировка вектора на каждом `insert` давала O(n² log n) на
/// построение таблицы, что заметно на geo-IP наборах в тысячи префиксов.
/// `finalize` вызывается один раз после массовой загрузки.
#[derive(Debug, Clone)]
pub struct IpTable<T> {
    v4_routes: Vec<RouteEntry<T>>,
    v6_routes: Vec<RouteEntry<T>>,
    /// Есть ли неотсортированные вставки после последнего finalize
    dirty: bool,
}

impl<T> Default for IpTable<T> {
    fn default() -> Self {
        Self {
            v4_routes: Vec::new(),
            v6_routes: Vec::new(),
            dirty: false,
        }
    }
}

impl<T: Clone> IpTable<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Вставить подсеть CIDR с ассоциированным значением.
    ///
    /// Сортировка откладывается до `finalize()`: вызывать `find` до него
    /// корректно только при отсутствии вставок (см. `finalize`).
    pub fn insert(&mut self, net: IpNet, value: T) {
        let entry = RouteEntry { net, value };
        match net {
            IpNet::V4(_) => self.v4_routes.push(entry),
            IpNet::V6(_) => self.v6_routes.push(entry),
        }
        self.dirty = true;
    }

    /// Привести таблицу в состояние, пригодное для поиска: сортировка по
    /// убыванию длины префикса (LPM: /32 → /24 → /16 → /8).
    ///
    /// Идемпотентна: повторный вызов без новых вставок ничего не делает.
    pub fn finalize(&mut self) {
        if !self.dirty {
            return;
        }
        self.v4_routes
            .sort_by_key(|a| std::cmp::Reverse(a.net.prefix_len()));
        self.v6_routes
            .sort_by_key(|a| std::cmp::Reverse(a.net.prefix_len()));
        self.dirty = false;
    }

    /// Готовность таблицы к поиску (нет неотсортированных вставок).
    pub fn is_finalized(&self) -> bool {
        !self.dirty
    }

    /// Поиск наиболее специфичного правила для IP-адреса.
    ///
    /// Требует предварительного вызова `finalize()` после массовых вставок —
    /// иначе наиболее специфичный префикс может быть найден неверно.
    pub fn find(&self, ip: IpAddr) -> Option<&T> {
        match ip {
            IpAddr::V4(addr) => self.find_v4(addr),
            IpAddr::V6(addr) => self.find_v6(addr),
        }
    }

    pub fn find_v4(&self, ip: Ipv4Addr) -> Option<&T> {
        for entry in &self.v4_routes {
            if let IpNet::V4(ref v4_net) = entry.net {
                if v4_net.contains(&ip) {
                    return Some(&entry.value);
                }
            }
        }
        None
    }

    pub fn find_v6(&self, ip: Ipv6Addr) -> Option<&T> {
        for entry in &self.v6_routes {
            if let IpNet::V6(ref v6_net) = entry.net {
                if v6_net.contains(&ip) {
                    return Some(&entry.value);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_longest_prefix_match() {
        let mut table = IpTable::new();
        table.insert(IpNet::from_str("10.0.0.0/8").unwrap(), "broad_10");
        table.insert(IpNet::from_str("10.1.0.0/16").unwrap(), "specific_10_1");
        table.insert(IpNet::from_str("10.1.2.3/32").unwrap(), "exact_host");
        table.finalize();

        let ip_host = IpAddr::from_str("10.1.2.3").unwrap();
        assert_eq!(table.find(ip_host), Some(&"exact_host"));

        let ip_subnet = IpAddr::from_str("10.1.5.99").unwrap();
        assert_eq!(table.find(ip_subnet), Some(&"specific_10_1"));

        let ip_broad = IpAddr::from_str("10.99.0.1").unwrap();
        assert_eq!(table.find(ip_broad), Some(&"broad_10"));

        let ip_other = IpAddr::from_str("192.168.1.1").unwrap();
        assert_eq!(table.find(ip_other), None);
    }

    /// Сортировка откладывается до finalize(): порядок вставки не должен влиять
    /// на результат LPM. Ранее пересортировка выполнялась на каждой вставке —
    /// это давало O(n² log n) при массовой загрузке geo-IP наборов.
    #[test]
    fn test_lpm_is_order_independent_after_finalize() {
        let mut table = IpTable::new();
        // Специфичный префикс вставлен ПЕРВЫМ — при отсутствии сортировки он
        // не был бы найден при поиске.
        table.insert(IpNet::from_str("10.1.2.3/32").unwrap(), "exact_host");
        table.insert(IpNet::from_str("10.1.0.0/16").unwrap(), "specific_10_1");
        table.insert(IpNet::from_str("10.0.0.0/8").unwrap(), "broad_10");
        table.finalize();

        let ip_host = IpAddr::from_str("10.1.2.3").unwrap();
        assert_eq!(table.find(ip_host), Some(&"exact_host"));
    }

    #[test]
    fn test_finalize_is_idempotent() {
        let mut table = IpTable::new();
        table.insert(IpNet::from_str("10.0.0.0/8").unwrap(), "broad_10");
        assert!(!table.is_finalized());

        table.finalize();
        assert!(table.is_finalized());

        // Повторный вызов не меняет состояние
        table.finalize();
        assert!(table.is_finalized());

        // Новая вставка снова помечает таблицу как требующую сортировки
        table.insert(IpNet::from_str("10.1.0.0/16").unwrap(), "specific");
        assert!(!table.is_finalized());
        table.finalize();

        let ip = IpAddr::from_str("10.1.5.99").unwrap();
        assert_eq!(table.find(ip), Some(&"specific"));
    }
}
