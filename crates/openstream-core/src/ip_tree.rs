use ipnet::IpNet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Запись в таблице маршрутизации IP
#[derive(Debug, Clone)]
struct RouteEntry<T> {
    net: IpNet,
    value: T,
}

/// Таблица сопоставления IP-адресов по принципу Longest Prefix Match (LPM)
#[derive(Debug, Clone)]
pub struct IpTable<T> {
    v4_routes: Vec<RouteEntry<T>>,
    v6_routes: Vec<RouteEntry<T>>,
}

impl<T> Default for IpTable<T> {
    fn default() -> Self {
        Self {
            v4_routes: Vec::new(),
            v6_routes: Vec::new(),
        }
    }
}

impl<T: Clone> IpTable<T> {
    pub fn new() -> Self {
        Self {
            v4_routes: Vec::new(),
            v6_routes: Vec::new(),
        }
    }

    /// Вставить подсеть CIDR с ассоциированным значением
    pub fn insert(&mut self, net: IpNet, value: T) {
        let entry = RouteEntry { net, value };
        match net {
            IpNet::V4(_) => {
                self.v4_routes.push(entry);
                // Сортировка по убыванию префикса (LPM): /32 -> /24 -> /16 -> /8
                self.v4_routes
                    .sort_by(|a, b| b.net.prefix_len().cmp(&a.net.prefix_len()));
            }
            IpNet::V6(_) => {
                self.v6_routes.push(entry);
                self.v6_routes
                    .sort_by(|a, b| b.net.prefix_len().cmp(&a.net.prefix_len()));
            }
        }
    }

    /// Поиск наиболее специфичного правила для IP-адреса
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

        let ip_host = IpAddr::from_str("10.1.2.3").unwrap();
        assert_eq!(table.find(ip_host), Some(&"exact_host"));

        let ip_subnet = IpAddr::from_str("10.1.5.99").unwrap();
        assert_eq!(table.find(ip_subnet), Some(&"specific_10_1"));

        let ip_broad = IpAddr::from_str("10.99.0.1").unwrap();
        assert_eq!(table.find(ip_broad), Some(&"broad_10"));

        let ip_other = IpAddr::from_str("192.168.1.1").unwrap();
        assert_eq!(table.find(ip_other), None);
    }
}
