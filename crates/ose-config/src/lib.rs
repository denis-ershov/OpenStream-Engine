//! Конфигурация демона (YAML; зеркало UCI).

use std::fs;
use std::path::Path;

use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

/// Vec, который принимает missing / null / [] (uci2yaml мог писать `custom_domains:` → null).
fn deserialize_string_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<Vec<String>>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProxyMode {
    /// Playlist Edge: клиент тянет m3u8 с роутера (без CA); сегменты с CDN.
    #[default]
    Edge,
    /// Прозрачный MITM (nft redirect HLS IP → :18080). Legacy / advanced; нужен CA.
    Transparent,
    /// Явный HTTP(S) proxy (CONNECT) — отладка / legacy.
    Explicit,
    /// Алиас transparent (старое имя UCI).
    RedirectWhitelist,
    Off,
}

impl<'de> Deserialize<'de> for ProxyMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "edge" => Self::Edge,
            "transparent" => Self::Transparent,
            "explicit" => Self::Explicit,
            "redirect_whitelist" => Self::RedirectWhitelist,
            "off" => Self::Off,
            // неизвестное значение (старые/битые конфиги) → edge, не падаем
            _ => Self::Edge,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrefetchPolicyConfig {
    Keep,
    StripAll,
    #[default]
    StripWhenAdsRemoved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_listen")]
    pub listen: String,
    /// Публичный URL прокси для rewrite master (например http://192.168.1.1:18080).
    #[serde(default)]
    pub proxy_public_url: Option<String>,
    #[serde(default)]
    pub mode: ProxyMode,
    #[serde(default = "default_ttl")]
    pub cache_ttl_secs: u64,
    #[serde(default = "default_max_manifest")]
    pub max_manifest_bytes: usize,
    #[serde(default = "default_nft_file")]
    pub nft_file: String,
    /// Domains/IPs for nft set `openstream_hls` (transparent divert / legacy).
    #[serde(default = "default_hostlist_file")]
    pub hostlist_file: String,
    #[serde(default = "default_hostlist_refresh")]
    pub hostlist_refresh_secs: u64,
    /// Сервисы, чьи hostlist-файлы входят в compose (twitch, kick, …).
    #[serde(default = "default_hostlist_services", deserialize_with = "deserialize_string_vec")]
    pub hostlist_services: Vec<String>,
    /// Кастомные домены/IP (одна строка = один хост).
    #[serde(default, deserialize_with = "deserialize_string_vec")]
    pub custom_domains: Vec<String>,
    /// Тянуть hostlists с GitHub (или другого base URL).
    #[serde(default)]
    pub hostlist_remote: bool,
    #[serde(default = "default_hostlist_remote_base")]
    pub hostlist_remote_base: String,
    #[serde(default = "default_hostlist_remote_hours")]
    pub hostlist_remote_interval_hours: u64,
    /// MITM CA — только для transparent/explicit; для edge игнорируется.
    #[serde(default)]
    pub mitm: bool,
    #[serde(default)]
    pub prefetch_policy: PrefetchPolicyConfig,
    #[serde(default)]
    pub twitch: TwitchConfig,
    #[serde(default)]
    pub kick: ServiceToggle,
    #[serde(default)]
    pub trovo: ServiceToggle,
    #[serde(default)]
    pub youtube: ServiceToggle,
    #[serde(default = "default_dash")]
    pub dash: ServiceToggle,
    /// Доп. YAML с rulesets (универсальный rule engine).
    #[serde(default)]
    pub rules_file: Option<String>,
    #[serde(default)]
    pub observability: ObservabilityConfig,
    #[serde(default)]
    pub tls: TlsConfig,
}

fn default_listen() -> String {
    "0.0.0.0:18080".into()
}

fn default_ttl() -> u64 {
    2
}

fn default_max_manifest() -> usize {
    2 * 1024 * 1024
}

fn default_nft_file() -> String {
    "/usr/share/openstream/nft/openstream.nft".into()
}

fn default_hostlist_file() -> String {
    "/var/run/openstream/hostlist-effective.txt".into()
}

fn default_hostlist_refresh() -> u64 {
    300
}

fn default_hostlist_services() -> Vec<String> {
    vec!["twitch".into()]
}

fn default_hostlist_remote_base() -> String {
    "https://raw.githubusercontent.com/denis-ershov/OpenStream-Engine/main/package/openwrt/files/hostlists"
        .into()
}

fn default_hostlist_remote_hours() -> u64 {
    12
}

fn default_true() -> bool {
    true
}

impl ProxyMode {
    /// Режим с nft redirect + transparent TLS accept (legacy MITM).
    pub fn uses_transparent_divert(&self) -> bool {
        matches!(self, Self::Transparent | Self::RedirectWhitelist)
    }

    /// Playlist Edge / nested fetch / explicit proxy (не Off).
    pub fn allows_playlist_proxy(&self) -> bool {
        !matches!(self, Self::Off)
    }
}

fn default_dash() -> ServiceToggle {
    ServiceToggle {
        enabled: true,
        debug: false,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitchConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub detect_stitched: bool,
    #[serde(default = "default_true")]
    pub detect_daterange: bool,
    #[serde(default)]
    pub detect_regex: bool,
    #[serde(default = "default_max_wait")]
    pub max_wait_secs: u64,
    #[serde(default)]
    pub debug: bool,
    /// Opt-in scaffold для seamless backup/token (НЕ default; strip остаётся основным режимом).
    #[serde(default)]
    pub backup_seamless: bool,
}

fn default_max_wait() -> u64 {
    30
}

impl Default for TwitchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            detect_stitched: true,
            detect_daterange: true,
            detect_regex: false,
            max_wait_secs: 30,
            debug: false,
            backup_seamless: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceToggle {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub debug: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    #[serde(default = "default_true")]
    pub metrics: bool,
    #[serde(default = "default_true")]
    pub events: bool,
    #[serde(default = "default_event_capacity")]
    pub event_capacity: usize,
}

fn default_event_capacity() -> usize {
    128
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            metrics: true,
            events: true,
            event_capacity: default_event_capacity(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TlsConfig {
    pub ca_cert: Option<String>,
    pub ca_key: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen: default_listen(),
            proxy_public_url: None,
            mode: ProxyMode::default(),
            cache_ttl_secs: default_ttl(),
            max_manifest_bytes: default_max_manifest(),
            nft_file: default_nft_file(),
            hostlist_file: default_hostlist_file(),
            hostlist_refresh_secs: default_hostlist_refresh(),
            hostlist_services: default_hostlist_services(),
            custom_domains: Vec::new(),
            hostlist_remote: false,
            hostlist_remote_base: default_hostlist_remote_base(),
            hostlist_remote_interval_hours: default_hostlist_remote_hours(),
            mitm: false,
            prefetch_policy: PrefetchPolicyConfig::default(),
            twitch: TwitchConfig::default(),
            kick: ServiceToggle::default(),
            trovo: ServiceToggle::default(),
            youtube: ServiceToggle::default(),
            dash: ServiceToggle {
                enabled: true,
                debug: false,
            },
            rules_file: None,
            observability: ObservabilityConfig::default(),
            tls: TlsConfig::default(),
        }
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let text = fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&text)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_custom_domains_ok() {
        let y = r#"
listen: "0.0.0.0:18080"
mode: edge
hostlist_services:
  - twitch
custom_domains:
hostlist_remote: false
mitm: false
"#;
        let c: Config = serde_yaml::from_str(y).expect("null custom_domains must parse");
        assert!(c.custom_domains.is_empty());
        assert_eq!(c.mode, ProxyMode::Edge);
        assert_eq!(c.hostlist_services, vec!["twitch".to_string()]);
    }

    #[test]
    fn empty_list_ok() {
        let y = "mode: edge\ncustom_domains: []\nhostlist_services: []\n";
        let c: Config = serde_yaml::from_str(y).unwrap();
        assert!(c.custom_domains.is_empty());
        assert!(c.hostlist_services.is_empty());
    }
}
