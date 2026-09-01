use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::capabilities::PlatformCapabilities;

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("Возможность '{0:?}' не поддерживается текущей платформой")]
    UnsupportedCapability(PlatformCapabilities),
    #[error("Ошибка сетевого стека ОС: {0}")]
    SystemError(String),
}

/// Информация о доступном исходящем шлюзе (Egress Gateway)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EgressGateway {
    pub id: String,
    pub tag: String,
    pub protocol: String,
    pub endpoint: String,
    pub healthy: bool,
}

/// Телеметрия и метрики бэкенда
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendMetrics {
    pub active_connections: u64,
    pub bytes_direct: u64,
    pub bytes_proxied: u64,
    pub bytes_blocked: u64,
    pub dns_queries_resolved: u64,
}

/// Скомпилированный набор правил, готовый к выгрузке в платформенный бэкенд
#[derive(Debug, Clone, Default)]
pub struct CompiledRuleSet {
    pub direct_domains: Vec<String>,
    pub dpi_evasive_domains: Vec<String>,
    pub zapret2_domains: Vec<(String, String)>, // (domain, preset)
    pub streamproxy_domains: Vec<String>,
    pub proxy_domains: Vec<(String, String)>, // (domain, gateway_id)
    pub blocked_domains: Vec<String>,
    pub dns_overrides: Vec<(String, Vec<std::net::IpAddr>)>,
    pub bypass_cidrs: Vec<ipnet::IpNet>,
    pub bypass_p2p: bool,
}

/// Универсальный контракт платформенного сетевого адаптера
#[async_trait]
pub trait NetworkBackend: Send + Sync {
    /// Физические возможности платформы (OpenWrt vs iOS vs Android)
    fn capabilities(&self) -> PlatformCapabilities;

    /// Применить скомпилированный набор правил в сетевую подсистему ОС
    async fn apply_ruleset(&mut self, compiled: &CompiledRuleSet) -> Result<(), BackendError>;

    /// Обновить перечень доступных туннелей/шлюзов
    async fn update_egress_gateways(&mut self, gateways: &[EgressGateway]) -> Result<(), BackendError>;

    /// Сбор телеметрии состояния адаптера
    async fn collect_metrics(&self) -> BackendMetrics;
}
