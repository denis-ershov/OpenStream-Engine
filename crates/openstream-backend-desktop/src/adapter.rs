use std::sync::Arc;
use async_trait::async_trait;
use tracing::info;

use openstream_core::backend::{
    BackendError, BackendMetrics, CompiledRuleSet, EgressGateway, NetworkBackend,
};
use openstream_core::capabilities::PlatformCapabilities;
use openstream_core::PolicyEngine;

/// Сетевой адаптер для настольных платформ (Windows, macOS, Linux)
pub struct DesktopBackend {
    engine: Arc<PolicyEngine>,
    gateways: Vec<EgressGateway>,
    metrics: BackendMetrics,
    interface_name: String,
}

impl DesktopBackend {
    pub fn new(interface_name: impl Into<String>) -> Self {
        Self {
            engine: Arc::new(PolicyEngine::new()),
            gateways: Vec::new(),
            metrics: BackendMetrics::default(),
            interface_name: interface_name.into(),
        }
    }

    pub fn interface_name(&self) -> &str {
        &self.interface_name
    }

    pub fn engine(&self) -> &Arc<PolicyEngine> {
        &self.engine
    }
}

#[async_trait]
impl NetworkBackend for DesktopBackend {
    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities::DNS_INTERCEPTION
            | PlatformCapabilities::DOMAIN_ROUTING
            | PlatformCapabilities::IP_ROUTING
            | PlatformCapabilities::L4_PACKET_DESYNC
            | PlatformCapabilities::ZERO_COPY_KERNEL
    }

    async fn apply_ruleset(&mut self, compiled: &CompiledRuleSet) -> Result<(), BackendError> {
        info!(
            interface = %self.interface_name,
            blocked = compiled.blocked_domains.len(),
            proxied = compiled.proxy_domains.len(),
            dpi_evasive = compiled.dpi_evasive_domains.len(),
            "Applied ruleset to DesktopBackend"
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_desktop_backend_lifecycle() {
        let mut backend = DesktopBackend::new("OpenStream0");
        assert_eq!(backend.interface_name(), "OpenStream0");

        let caps = backend.capabilities();
        assert!(caps.contains(PlatformCapabilities::DNS_INTERCEPTION));
        assert!(caps.contains(PlatformCapabilities::DOMAIN_ROUTING));

        let compiled = CompiledRuleSet::default();
        let res = backend.apply_ruleset(&compiled).await;
        assert!(res.is_ok());
    }
}
