//! Модуль мониторинга маршрутизации и активных соединений в реальном времени
//! Предоставляет учет трафика, доменов и секций с фиксированным кольцевым буфером

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Секция / сетевой движок, через который направляется трафик
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingSection {
    /// Анти-DPI десинхронизация пакетов через Zapret2 (nfqws2)
    Zapret2,
    /// Локальная модификация HLS/DASH плейлистов в StreamProxy (:8888)
    StreamProxy,
    /// Проксирование через зашифрованный туннель sing-box / WireGuard
    SingBox,
    /// Приоритетное исключение из списков Zapret2 / VPN
    Bypass,
    /// Прямой провайдерский интернет (WAN Direct)
    Direct,
    /// Блокировка на уровне DNS (Sinkhole 0.0.0.0 / ::)
    Block,
}

impl RoutingSection {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Zapret2 => "zapret2",
            Self::StreamProxy => "streamproxy",
            Self::SingBox => "singbox",
            Self::Bypass => "bypass",
            Self::Direct => "direct",
            Self::Block => "block",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Zapret2 => "Zapret2 (nfqws2)",
            Self::StreamProxy => "StreamProxy (:8888)",
            Self::SingBox => "sing-box / VPN",
            Self::Bypass => "Bypass (Пропуск / Исключение)",
            Self::Direct => "Direct (WAN Direct)",
            Self::Block => "Block (DNS Sinkhole)",
        }
    }

    pub fn badge_color(&self) -> &'static str {
        match self {
            Self::Zapret2 => "#10b981",    // Emerald
            Self::StreamProxy => "#38bdf8",// Sky Blue
            Self::SingBox => "#a855f7",    // Purple
            Self::Bypass => "#06b6d4",     // Cyan
            Self::Direct => "#94a3b8",     // Slate
            Self::Block => "#f43f5e",      // Rose Red
        }
    }
}

/// Запись о маршрутизированном потоке / соединении
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowRecord {
    pub id: String,
    pub client_ip: String,
    pub client_mac: Option<String>,
    pub client_name: Option<String>,
    pub target_domain: String,
    pub target_ip: Option<String>,
    pub section: RoutingSection,
    pub details: String,
    pub bytes: u64,
    pub packets: u64,
    pub timestamp_sec: u64,
}

/// Потокобезопасный трекер соединений с ограниченным кольцевым буфером
pub struct FlowTracker {
    max_capacity: usize,
    buffer: RwLock<VecDeque<FlowRecord>>,
}

impl FlowTracker {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            max_capacity: max_capacity.clamp(50, 1000),
            buffer: RwLock::new(VecDeque::with_capacity(max_capacity)),
        }
    }

    /// Добавление новой записи о потоке с вытеснением устаревших при переполнении
    pub fn record_flow(&self, mut flow: FlowRecord) {
        if flow.timestamp_sec == 0 {
            flow.timestamp_sec = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
        }

        let mut buf = self.buffer.write().unwrap();
        if buf.len() >= self.max_capacity {
            buf.pop_front();
        }
        buf.push_back(flow);
    }

    /// Получение снимка активных потоков (от новых к старым)
    pub fn get_flows(&self) -> Vec<FlowRecord> {
        let buf = self.buffer.read().unwrap();
        buf.iter().rev().cloned().collect()
    }

    /// Очистка накопленной истории
    pub fn clear(&self) {
        let mut buf = self.buffer.write().unwrap();
        buf.clear();
    }

    /// Количество зарегистрированных потоков в буфере
    pub fn count(&self) -> usize {
        self.buffer.read().unwrap().len()
    }
}

impl Default for FlowTracker {
    fn default() -> Self {
        Self::new(200)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_section_properties() {
        assert_eq!(RoutingSection::Zapret2.as_str(), "zapret2");
        assert_eq!(RoutingSection::StreamProxy.as_str(), "streamproxy");
        assert_eq!(RoutingSection::SingBox.as_str(), "singbox");
        assert_eq!(RoutingSection::Block.as_str(), "block");

        assert_eq!(RoutingSection::Zapret2.badge_color(), "#10b981");
        assert!(RoutingSection::SingBox.display_name().contains("sing-box"));
    }

    #[test]
    fn test_flow_tracker_bounded_buffer() {
        let tracker = FlowTracker::new(50);
        assert_eq!(tracker.count(), 0);

        for i in 0..60 {
            tracker.record_flow(FlowRecord {
                id: format!("flow-{}", i),
                client_ip: "192.168.1.100".into(),
                client_mac: Some("AA:BB:CC:DD:EE:FF".into()),
                client_name: Some("iPhone-User".into()),
                target_domain: format!("site-{}.com", i),
                target_ip: Some("1.2.3.4".into()),
                section: RoutingSection::Zapret2,
                details: "Preset youtube_4k".into(),
                bytes: 1024,
                packets: 8,
                timestamp_sec: 1700000000 + i,
            });
        }

        // Проверяем, что размер ограничен 50 и старые записи были вытеснены
        assert_eq!(tracker.count(), 50);
        let flows = tracker.get_flows();
        assert_eq!(flows[0].id, "flow-59"); // Самая новая в начале
        assert_eq!(flows[49].id, "flow-10"); // Самая старая из оставшихся

        tracker.clear();
        assert_eq!(tracker.count(), 0);
    }
}
