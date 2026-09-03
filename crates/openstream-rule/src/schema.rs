use std::collections::BTreeMap;
use std::net::IpAddr;
use ipnet::IpNet;
use serde::{Deserialize, Serialize};

/// Версия схемы манифеста правил
pub const CURRENT_SCHEMA_VERSION: &str = "2.1";

/// Корневой манифест правила (.osrule)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleManifest {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub capabilities_required: Vec<String>,
    /// SecOps-флаг: разрешить перехват системных/критических доменов
    #[serde(default)]
    pub allow_sensitive: bool,
    /// Список правил сопоставления трафика
    #[serde(default)]
    pub matches: Vec<MatchRule>,
    /// Словарь именованных стратегий маршрутизации
    #[serde(default)]
    pub strategies: BTreeMap<String, StrategyDef>,
}

fn default_schema_version() -> String {
    CURRENT_SCHEMA_VERSION.to_string()
}

fn default_true() -> bool {
    true
}

/// Фильтр клиентов локальной сети (Per-Device Routing)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ClientFilter {
    /// Включенные MAC-адреса устройств
    #[serde(default)]
    pub include_macs: Vec<String>,
    /// Включенные IP-адреса устройств
    #[serde(default)]
    pub include_ips: Vec<IpAddr>,
    /// Исключенные MAC-адреса устройств
    #[serde(default)]
    pub exclude_macs: Vec<String>,
    /// Исключенные IP-адреса устройств
    #[serde(default)]
    pub exclude_ips: Vec<IpAddr>,
}

/// Конфигурация исключений из маршрутизации (Bypass)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BypassConfig {
    /// Исключенные IP-адреса и подсети (решение issue #88)
    #[serde(default)]
    pub bypass_cidrs: Vec<IpNet>,
    /// Пускать торренты / P2P напрямую в WAN (решение issue #72)
    #[serde(default)]
    pub bypass_p2p: bool,
    /// Блокировка QUIC (UDP 443) для форсирования TCP TLS 1.3 (решение проблем YouTube/Chrome)
    #[serde(default)]
    pub disable_quic: bool,
    /// Блокировка прямого браузерного DoH/DoT в обход dnsmasq (порт 853)
    #[serde(default)]
    pub block_doh: bool,
    /// Исключение NTP (UDP 123) для точного системного времени
    #[serde(default)]
    pub exclude_ntp: bool,
}

/// Правило сопоставления трафика (Matcher)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MatchRule {
    pub group: Option<String>,
    /// Фильтр клиентских устройств в LAN (Per-Device Routing)
    #[serde(default)]
    pub clients: Option<ClientFilter>,
    /// Исключения из маршрутизации (Bypass)
    #[serde(default)]
    pub bypass: Option<BypassConfig>,
    /// Список доменов или wildcard-шаблонов (*.example.com, example.com)
    #[serde(default)]
    pub domains: Vec<String>,
    /// Список IP-сетей / CIDR
    #[serde(default)]
    pub cidrs: Vec<IpNet>,
    /// Порты назначения L4 (TCP/UDP)
    #[serde(default)]
    pub ports: Vec<u16>,
    /// Идентификаторы приложений (например, com.amazon.twitch на Android/macOS)
    #[serde(default)]
    pub app_ids: Vec<String>,
    /// Ссылка на именованную стратегию из секции strategies
    pub strategy: Option<String>,
    /// Прямое действие (если стратегия не используется)
    pub action: Option<RoutingAction>,
}

/// Именованная стратегия с цепочкой предпочтений (fallback-цепочка)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyDef {
    /// Список шлюзов/способов выхода по убыванию приоритета
    /// Примеры: ["geo:al", "proxy:clean_vpn", "smartdns:eu", "direct"]
    #[serde(default)]
    pub preference: Vec<String>,
}

/// Итоговое действие маршрутизации трафика
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum RoutingAction {
    Simple(SimpleAction),
    Detailed(DetailedAction),
}

impl RoutingAction {
    pub fn is_direct(&self) -> bool {
        matches!(
            self,
            RoutingAction::Simple(SimpleAction::Direct)
                | RoutingAction::Detailed(DetailedAction::Tagged(TaggedAction::Direct))
        )
    }

    pub fn is_bypass(&self) -> bool {
        matches!(
            self,
            RoutingAction::Simple(SimpleAction::Bypass | SimpleAction::Pass)
                | RoutingAction::Detailed(DetailedAction::Tagged(TaggedAction::Bypass | TaggedAction::Pass))
        )
    }

    pub fn is_block(&self) -> bool {
        matches!(
            self,
            RoutingAction::Simple(SimpleAction::Block)
                | RoutingAction::Detailed(DetailedAction::Tagged(TaggedAction::Block))
        )
    }

    pub fn is_dpi_evasive(&self) -> bool {
        matches!(
            self,
            RoutingAction::Simple(SimpleAction::DpiEvasiveDirect)
                | RoutingAction::Detailed(DetailedAction::Tagged(TaggedAction::DpiEvasiveDirect))
        )
    }

    pub fn is_zapret2(&self) -> bool {
        matches!(
            self,
            RoutingAction::Simple(SimpleAction::Zapret2)
                | RoutingAction::Detailed(DetailedAction::Tagged(TaggedAction::Zapret2 { .. }))
                | RoutingAction::Detailed(DetailedAction::Nested(NestedAction::Zapret2 { .. }))
        )
    }

    pub fn is_stream_proxy(&self) -> bool {
        matches!(
            self,
            RoutingAction::Simple(SimpleAction::StreamProxy)
                | RoutingAction::Detailed(DetailedAction::Tagged(TaggedAction::StreamProxy { .. }))
                | RoutingAction::Detailed(DetailedAction::Nested(NestedAction::StreamProxy { .. }))
        )
    }
}

/// Базовое действие без дополнительных параметров
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SimpleAction {
    Direct,
    Bypass,
    #[serde(alias = "pass", alias = "exclude")]
    Pass,
    DpiEvasiveDirect,
    Zapret2,
    StreamProxy,
    Block,
}

/// Детализированное действие с параметрами
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum DetailedAction {
    Tagged(TaggedAction),
    Nested(NestedAction),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaggedAction {
    Direct,
    Bypass,
    #[serde(alias = "pass", alias = "exclude")]
    Pass,
    DpiEvasiveDirect,
    Zapret2 {
        preset: String,
        #[serde(default)]
        custom_args: Option<String>,
    },
    StreamProxy {
        mode: String,
    },
    Block,
    Proxy {
        gateway_id: String,
    },
    DnsOverride {
        ips: Vec<IpAddr>,
    },
    StripPayload {
        plugin_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NestedAction {
    Zapret2 {
        preset: String,
        #[serde(default)]
        custom_args: Option<String>,
    },
    StreamProxy {
        mode: String,
    },
    Proxy {
        gateway_id: String,
    },
    DnsOverride {
        ips: Vec<IpAddr>,
    },
    StripPayload {
        plugin_id: String,
    },
}



