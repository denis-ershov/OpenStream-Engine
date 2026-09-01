use std::fs;
use std::path::PathBuf;
use async_trait::async_trait;
use tracing::info;

use openstream_core::backend::{
    BackendError, BackendMetrics, CompiledRuleSet, EgressGateway, NetworkBackend,
};
use openstream_core::capabilities::PlatformCapabilities;
use crate::dnsmasq::generate_dnsmasq_config;
use crate::nftables::generate_nftables_sets;

/// Сетевой адаптер OpenWrt, управляющий dnsmasq и таблицами nftables
pub struct OpenWrtBackend {
    dnsmasq_conf_path: PathBuf,
    nft_conf_path: PathBuf,
    table_family: String,
    table_name: String,
    gateways: Vec<EgressGateway>,
    metrics: BackendMetrics,
}

impl OpenWrtBackend {
    pub fn new(dnsmasq_conf_path: impl Into<PathBuf>, nft_conf_path: impl Into<PathBuf>) -> Self {
        Self {
            dnsmasq_conf_path: dnsmasq_conf_path.into(),
            nft_conf_path: nft_conf_path.into(),
            table_family: "inet".into(),
            table_name: "openstream".into(),
            gateways: Vec::new(),
            metrics: BackendMetrics::default(),
        }
    }

    pub fn with_table(mut self, family: &str, name: &str) -> Self {
        self.table_family = family.to_string();
        self.table_name = name.to_string();
        self
    }
}

#[async_trait]
impl NetworkBackend for OpenWrtBackend {
    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities::DNS_INTERCEPTION
            | PlatformCapabilities::DOMAIN_ROUTING
            | PlatformCapabilities::IP_ROUTING
            | PlatformCapabilities::L7_PAYLOAD_STRIP
            | PlatformCapabilities::L4_PACKET_DESYNC
            | PlatformCapabilities::ZERO_COPY_KERNEL
    }

    async fn apply_ruleset(&mut self, compiled: &CompiledRuleSet) -> Result<(), BackendError> {
        info!("Applying compiled ruleset to OpenWrt backend");

        // 1. Генерация и запись dnsmasq конфигурации
        let dnsmasq_cfg = generate_dnsmasq_config(compiled, &self.table_family, &self.table_name);
        if let Some(parent) = self.dnsmasq_conf_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&self.dnsmasq_conf_path, dnsmasq_cfg)
            .map_err(|e| BackendError::SystemError(format!("Failed to write dnsmasq conf: {}", e)))?;

        // 2. Генерация и запись nftables конфигурации
        let nft_cfg = generate_nftables_sets(compiled, &self.table_family, &self.table_name);
        if let Some(parent) = self.nft_conf_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&self.nft_conf_path, nft_cfg)
            .map_err(|e| BackendError::SystemError(format!("Failed to write nftables conf: {}", e)))?;

        Ok(())
    }

    async fn update_egress_gateways(&mut self, gateways: &[EgressGateway]) -> Result<(), BackendError> {
        self.gateways = gateways.to_vec();
        Ok(())
    }

    async fn collect_metrics(&self) -> BackendMetrics {
        self.metrics.clone()
    }
}
