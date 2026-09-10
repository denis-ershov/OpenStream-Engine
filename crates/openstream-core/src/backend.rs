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
#[derive(Debug, Clone)]
pub struct CompiledRuleSet {
    pub direct_domains: Vec<String>,
    pub bypass_domains: Vec<String>,
    pub dpi_evasive_domains: Vec<String>,
    pub zapret2_domains: Vec<(String, String)>, // (domain, preset)
    pub streamproxy_domains: Vec<String>,
    pub proxy_domains: Vec<(String, String)>, // (domain, gateway_id)
    pub blocked_domains: Vec<String>,
    pub dns_overrides: Vec<(String, Vec<std::net::IpAddr>)>,
    pub bypass_cidrs: Vec<ipnet::IpNet>,
    pub bypass_clients: Vec<String>,
    pub bypass_p2p: bool,
    pub disable_quic: bool,
    pub block_doh: bool,
    pub exclude_ntp: bool,
    pub failover_threshold: u32,
    pub failover_recovery_sec: u64,
    pub doh_client_cert: Option<String>,
    pub doh_client_key: Option<String>,
    pub ipv4_only: bool,
}

impl Default for CompiledRuleSet {
    fn default() -> Self {
        Self {
            direct_domains: Vec::new(),
            bypass_domains: Vec::new(),
            dpi_evasive_domains: Vec::new(),
            zapret2_domains: Vec::new(),
            streamproxy_domains: Vec::new(),
            proxy_domains: Vec::new(),
            blocked_domains: Vec::new(),
            dns_overrides: Vec::new(),
            bypass_cidrs: Vec::new(),
            bypass_clients: Vec::new(),
            bypass_p2p: true,
            disable_quic: false,
            block_doh: false,
            exclude_ntp: true,
            failover_threshold: 3,
            failover_recovery_sec: 30,
            doh_client_cert: None,
            doh_client_key: None,
            ipv4_only: false,
        }
    }
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
