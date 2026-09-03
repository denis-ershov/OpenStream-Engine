use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use openstream_core::engine::RoutingVerdict;
use openstream_core::PolicyEngine;
use openstream_rule::schema::RuleManifest;
use openstream_rule::{load_rule_file, parse_rule_yaml};

use crate::models::{MobileError, MobileMetrics, MobileRuleInfo, MobileVerdict};

#[derive(uniffi::Object)]
pub struct MobileEngine {
    engine: Arc<PolicyEngine>,
    manifests: RwLock<Vec<RuleManifest>>,
    // Атомарные метрики для real-time дашборда
    total_queries: AtomicU64,
    blocked_queries: AtomicU64,
    proxied_queries: AtomicU64,
    dpi_evasive_queries: AtomicU64,
    direct_queries: AtomicU64,
}

#[uniffi::export]
impl MobileEngine {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            engine: Arc::new(PolicyEngine::new()),
            manifests: RwLock::new(Vec::new()),
            total_queries: AtomicU64::new(0),
            blocked_queries: AtomicU64::new(0),
            proxied_queries: AtomicU64::new(0),
            dpi_evasive_queries: AtomicU64::new(0),
            direct_queries: AtomicU64::new(0),
        })
    }

    /// Загрузка правила из файла по пути
    pub fn load_rule_file(&self, path: String) -> Result<String, MobileError> {
        let rule = load_rule_file(&path).map_err(|e| match e {
            openstream_rule::RuleError::Io(err) => MobileError::FileNotFound(err.to_string()),
            openstream_rule::RuleError::Validation(val_err) => {
                MobileError::SecurityViolation(format!("{:?}", val_err))
            }
            other => MobileError::InvalidManifest(other.to_string()),
        })?;

        let id = rule.id.clone();
        self.engine.add_rule(rule.clone());

        if let Ok(mut lock) = self.manifests.write() {
            lock.retain(|m| m.id != id);
            lock.push(rule);
        }

        Ok(id)
    }

    /// Загрузка правила из YAML-строки (из App Group или сети)
    pub fn load_rule_manifest(&self, yaml: String) -> Result<String, MobileError> {
        let rule = parse_rule_yaml(&yaml).map_err(|e| match e {
            openstream_rule::RuleError::Validation(val_err) => {
                MobileError::SecurityViolation(format!("{:?}", val_err))
            }
            other => MobileError::InvalidManifest(other.to_string()),
        })?;

        let id = rule.id.clone();
        self.engine.add_rule(rule.clone());

        if let Ok(mut lock) = self.manifests.write() {
            lock.retain(|m| m.id != id);
            lock.push(rule);
        }

        Ok(id)
    }

    /// Рекурсивная загрузка всех .osrule.yaml правил из директории общего контейнера
    pub fn load_rules_dir(&self, dir_path: String) -> Result<u32, MobileError> {
        let p = Path::new(&dir_path);
        if !p.exists() || !p.is_dir() {
            return Err(MobileError::FileNotFound(format!(
                "Directory does not exist: {}",
                dir_path
            )));
        }

        let mut loaded = 0;
        let read_dir = fs::read_dir(p)
            .map_err(|e| MobileError::InternalError(format!("Read dir failed: {}", e)))?;

        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(sub_read) = fs::read_dir(&path) {
                    for sub_entry in sub_read.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.is_file()
                            && sub_path.to_string_lossy().ends_with(".osrule.yaml")
                        {
                            if let Ok(id) = self.load_rule_file(sub_path.to_string_lossy().to_string())
                            {
                                tracing::info!(rule_id = %id, "Loaded mobile rule");
                                loaded += 1;
                            }
                        }
                    }
                }
            } else if path.is_file() && path.to_string_lossy().ends_with(".osrule.yaml") {
                if let Ok(id) = self.load_rule_file(path.to_string_lossy().to_string()) {
                    tracing::info!(rule_id = %id, "Loaded mobile rule");
                    loaded += 1;
                }
            }
        }

        Ok(loaded)
    }

    /// Сопоставление доменного имени за O(k) без аллокаций в hot-path
    pub fn match_domain(&self, domain: String) -> MobileVerdict {
        self.total_queries.fetch_add(1, Ordering::Relaxed);
        let verdict = self.engine.resolve_domain(&domain, None);

        match verdict {
            RoutingVerdict::Direct => {
                self.direct_queries.fetch_add(1, Ordering::Relaxed);
                MobileVerdict::Direct
            }
            RoutingVerdict::Bypass => {
                self.direct_queries.fetch_add(1, Ordering::Relaxed);
                MobileVerdict::Bypass
            }
            RoutingVerdict::DpiEvasiveDirect => {
                self.dpi_evasive_queries.fetch_add(1, Ordering::Relaxed);
                MobileVerdict::DpiEvasiveDirect
            }
            RoutingVerdict::Proxy { gateway_id } => {
                self.proxied_queries.fetch_add(1, Ordering::Relaxed);
                MobileVerdict::Proxy { gateway_id }
            }
            RoutingVerdict::DnsOverride { ips } => {
                let str_ips = ips.into_iter().map(|ip| ip.to_string()).collect();
                MobileVerdict::DnsOverride { ips: str_ips }
            }
            RoutingVerdict::Block { reason } => {
                self.blocked_queries.fetch_add(1, Ordering::Relaxed);
                MobileVerdict::Block { reason }
            }
            RoutingVerdict::Zapret2 { preset, .. } => {
                self.dpi_evasive_queries.fetch_add(1, Ordering::Relaxed);
                MobileVerdict::Zapret2 { preset }
            }
            RoutingVerdict::StreamProxy { mode } => {
                MobileVerdict::StreamProxy { mode }
            }
            RoutingVerdict::StripPayload { .. } => {
                // Для мобильного TUN плейлист проксируется через локальный обработчик
                MobileVerdict::Direct
            }
        }
    }

    /// Список установленных правил
    pub fn get_installed_rules(&self) -> Vec<MobileRuleInfo> {
        let manifests = self.manifests.read().unwrap();
        manifests
            .iter()
            .map(|m| MobileRuleInfo {
                id: m.id.clone(),
                name: m.name.clone(),
                version: m.version.clone(),
                author: m.author.clone(),
                description: m.description.clone(),
                enabled: true,
                match_count: m.matches.len() as u32,
            })
            .collect()
    }

    /// Получение текущих метрик и расхода памяти
    pub fn get_metrics(&self) -> MobileMetrics {
        // Оценка памяти структур в RAM (обычно 1-2 МБ)
        let estimated_mem = 1024 * 1024 * 2; // ~2 MB

        MobileMetrics {
            total_queries: self.total_queries.load(Ordering::Relaxed),
            blocked_queries: self.blocked_queries.load(Ordering::Relaxed),
            proxied_queries: self.proxied_queries.load(Ordering::Relaxed),
            dpi_evasive_queries: self.dpi_evasive_queries.load(Ordering::Relaxed),
            direct_queries: self.direct_queries.load(Ordering::Relaxed),
            memory_bytes: estimated_mem,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_engine_lifecycle() {
        let engine = MobileEngine::new();

        let yaml = r#"
schema_version: "2.0"
id: "org.openstream.mobile.test"
name: "Mobile Test Rule"
version: "1.0.0"

matches:
  - domains: ["*.ads.test.com"]
    action: "block"
  - domains: ["video.stream.tv"]
    action: "direct"
  - domains: ["sni.bypass.org"]
    action: "dpi_evasive_direct"
"#;

        let id = engine
            .load_rule_manifest(yaml.to_string())
            .expect("Загрузка валидного правила должна пройти");
        assert_eq!(id, "org.openstream.mobile.test");

        // 1. Блокировка рекламы
        let v1 = engine.match_domain("tracker.ads.test.com".into());
        assert_eq!(v1, MobileVerdict::Block { reason: None });

        // 2. Direct видео
        let v2 = engine.match_domain("video.stream.tv".into());
        assert_eq!(v2, MobileVerdict::Direct);

        // 3. Anti-DPI
        let v3 = engine.match_domain("sni.bypass.org".into());
        assert_eq!(v3, MobileVerdict::DpiEvasiveDirect);

        // 4. Метрики
        let metrics = engine.get_metrics();
        assert_eq!(metrics.total_queries, 3);
        assert_eq!(metrics.blocked_queries, 1);
        assert_eq!(metrics.direct_queries, 1);
        assert_eq!(metrics.dpi_evasive_queries, 1);

        let rules = engine.get_installed_rules();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "Mobile Test Rule");
    }
}
