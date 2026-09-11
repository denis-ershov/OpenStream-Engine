//! Модуль интеграции и управления вариантами sing-box
//! Поддерживает 4 варианта: Stable, Extended, Tiny, ExtendedCompress

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Варианты сборки sing-box для OpenWrt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SingBoxVariant {
    /// Официальный стабильный пакет из репозиториев OpenWrt feeds
    Stable,
    /// Расширенная сборка с поддержкой xHTTP, TUIC, Reality, Shadowsocks 2022
    Extended,
    /// Минимальная облегченная сборка для роутеров с 64–128 МБ RAM (<8 МБ Flash)
    Tiny,
    /// Сжатая UPX расширенная сборка для экономии до 65% Flash-памяти
    ExtendedCompress,
}

impl SingBoxVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Extended => "extended",
            Self::Tiny => "tiny",
            Self::ExtendedCompress => "extended_compress",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Stable => "sing-box Stable (Официальная сборка OpenWrt)",
            Self::Extended => "sing-box Extended (xHTTP, TUIC, Reality, SS2022)",
            Self::Tiny => "sing-box Tiny (Облегченная для 64–128 МБ RAM)",
            Self::ExtendedCompress => "sing-box Extended Compress (UPX сжатие для малого Flash)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Stable => "Стандартная стабильная сборка из официального фида OpenWrt. Рекомендуется для большинства задач.",
            Self::Extended => "Включает новейшие транспорты (xHTTP/H2/H3), Shadowsocks 2022, TUIC v5 и Reality для надежного обхода блокировок.",
            Self::Tiny => "Урезаны второстепенные протоколы. Оптимизирована для 64–128 МБ RAM, размер файла <8 МБ.",
            Self::ExtendedCompress => "Полный функционал Extended, упакованный UPX. Занимает всего ~11 МБ во Flash-памяти.",
        }
    }
}

/// Информация об установленном бинарнике sing-box
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingBoxInfo {
    pub installed: bool,
    pub path: PathBuf,
    pub variant: Option<SingBoxVariant>,
    pub version: Option<String>,
    pub size_bytes: u64,
    pub supports_xhttp: bool,
    pub is_upx_compressed: bool,
}

/// Автоматическое определение установленного варианта sing-box и его характеристик
pub fn detect_singbox_info(custom_bin_path: Option<&Path>) -> SingBoxInfo {
    let bin_path = custom_bin_path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/bin/sing-box"));

    if !bin_path.exists() {
        return SingBoxInfo {
            installed: false,
            path: bin_path,
            variant: None,
            version: None,
            size_bytes: 0,
            supports_xhttp: false,
            is_upx_compressed: false,
        };
    }

    let size_bytes = fs::metadata(&bin_path).map(|m| m.len()).unwrap_or(0);

    // Проверка UPX заголовка (первые байты или сигнатура UPX!)
    let is_upx_compressed = check_upx_signature(&bin_path);

    // Опрос версии и возможностей через запуск бинарника
    let (version, supports_xhttp) = query_binary_capabilities(&bin_path);

    // Определение варианта на основе размера и поддержки xhttp
    let variant = if is_upx_compressed {
        Some(SingBoxVariant::ExtendedCompress)
    } else if supports_xhttp {
        Some(SingBoxVariant::Extended)
    } else if size_bytes > 0 && size_bytes < 15 * 1024 * 1024 {
        Some(SingBoxVariant::Tiny)
    } else {
        Some(SingBoxVariant::Stable)
    };

    SingBoxInfo {
        installed: true,
        path: bin_path,
        variant,
        version,
        size_bytes,
        supports_xhttp,
        is_upx_compressed,
    }
}

/// Проверка сигнатуры UPX в заголовке исполняемого файла
fn check_upx_signature(path: &Path) -> bool {
    if let Ok(data) = fs::read(path) {
        if data.windows(4).any(|w| w == b"UPX!") {
            return true;
        }
    }
    false
}

/// Опрос версии и параметров сборки sing-box
fn query_binary_capabilities(path: &Path) -> (Option<String>, bool) {
    let output = std::process::Command::new(path).arg("version").output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let first_line = stdout.lines().next().unwrap_or("").trim().to_string();
        let version = if first_line.is_empty() {
            None
        } else {
            Some(first_line)
        };

        let supports_xhttp = stdout.contains("with_xhttp")
            || stdout.contains("xhttp")
            || stdout.contains("with_reality");

        (version, supports_xhttp)
    } else {
        (None, false)
    }
}

/// Генерация безопасной конфигурации inbound для интеграции sing-box с OpenStream.
///
/// Важно:
/// * TPROXY-сокет обязан слушать `0.0.0.0`, а не `127.0.0.1`: ядро отдаёт ему пакеты
///   с чужим (удалённым) адресом назначения через `IP_TRANSPARENT`. На loopback
///   такой трафик не придёт, и перехват молча не работает.
/// * `route.default_mark` помечает исходящий трафик самого sing-box меткой
///   0x00880002, по которой правило `meta mark ... return` в `mangle_prerouting`
///   исключает его из повторного перехвата. Без этого возникает петля (решение #74).
pub fn generate_singbox_inbound_config(tproxy_port: u16) -> String {
    let routing_mark = crate::nftables::OPENSTREAM_SINGBOX_MARK;
    format!(
        r#"{{
  "inbounds": [
    {{
      "type": "tproxy",
      "tag": "openstream-tproxy-in",
      "listen": "0.0.0.0",
      "listen_port": {tproxy_port},
      "sniff": true,
      "sniff_override_destination": false
    }}
  ],
  "outbounds": [
    {{
      "type": "direct",
      "tag": "direct"
    }}
  ],
  "route": {{
    "auto_detect_interface": true,
    "default_mark": {routing_mark}
  }}
}}"#
    )
}

/// Генерация группы выбора узла по наименьшей задержке (URLTest / Best Latency Selector)
pub fn generate_singbox_urltest_outbound(
    tag: &str,
    outbounds: &[String],
    interval: &str,
    tolerance_ms: u32,
) -> String {
    let outbounds_json = outbounds
        .iter()
        .map(|o| format!(r#""{}""#, o))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"{{
  "type": "urltest",
  "tag": "{tag}",
  "outbounds": [{outbounds_json}],
  "url": "https://cp.cloudflare.com/generate_204",
  "interval": "{interval}",
  "tolerance": {tolerance_ms}
}}"#
    )
}

/// Генерация отказоустойчивой конфигурации Multi-DNS с Bootstrap-серверами
pub fn generate_singbox_dns_config(
    primary_dns: &[String],
    bootstrap_dns: &[String],
    prefer_ipv4: bool,
) -> String {
    let bootstrap_server = bootstrap_dns.first().map(|s| s.as_str()).unwrap_or("77.88.8.8");
    let strategy = if prefer_ipv4 { "prefer_ipv4" } else { "auto" };

    let mut servers = Vec::new();
    // 1. Добавляем основные DoH / DoT серверы с привязкой к bootstrap-резолверу
    for (idx, server_url) in primary_dns.iter().enumerate() {
        let tag = if idx == 0 { "dns-remote".to_string() } else { format!("dns-remote-backup-{}", idx) };
        servers.push(format!(
            r#"    {{
      "tag": "{tag}",
      "address": "{server_url}",
      "address_resolver": "dns-bootstrap",
      "strategy": "{strategy}"
    }}"#
        ));
    }

    // 2. Добавляем статический Bootstrap DNS для разрешения имен самих DoH-серверов
    servers.push(format!(
        r#"    {{
      "tag": "dns-bootstrap",
      "address": "{bootstrap_server}",
      "strategy": "ipv4_only",
      "detour": "direct"
    }}"#
    ));

    let servers_json = servers.join(",\n");

    format!(
        r#"{{
  "dns": {{
    "servers": [
{servers_json}
    ],
    "strategy": "{strategy}"
  }}
}}"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singbox_variants_properties() {
        assert_eq!(SingBoxVariant::Stable.as_str(), "stable");
        assert_eq!(SingBoxVariant::Extended.as_str(), "extended");
        assert_eq!(SingBoxVariant::Tiny.as_str(), "tiny");
        assert_eq!(SingBoxVariant::ExtendedCompress.as_str(), "extended_compress");

        assert!(SingBoxVariant::Extended.display_name().contains("xHTTP"));
        assert!(SingBoxVariant::Tiny.description().contains("64"));
    }

    #[test]
    fn test_generate_singbox_inbound_config() {
        let cfg = generate_singbox_inbound_config(10888);
        assert!(cfg.contains(r#""listen_port": 10888"#));
        assert!(cfg.contains(r#""tag": "openstream-tproxy-in""#));
        assert!(cfg.contains(r#""type": "tproxy""#));
    }

    /// TPROXY обязан слушать 0.0.0.0: пакеты приходят с чужим destination-адресом.
    #[test]
    fn test_tproxy_listens_on_wildcard_not_loopback() {
        let cfg = generate_singbox_inbound_config(10888);
        assert!(cfg.contains(r#""listen": "0.0.0.0""#));
        assert!(!cfg.contains(r#""listen": "127.0.0.1""#));
    }

    /// Без routing mark исходящий трафик sing-box снова попадёт в tproxy (петля #74).
    #[test]
    fn test_singbox_config_sets_routing_mark() {
        let cfg = generate_singbox_inbound_config(10888);
        assert!(cfg.contains(&format!(
            r#""default_mark": {}"#,
            crate::nftables::OPENSTREAM_SINGBOX_MARK
        )));
    }

    #[test]
    fn test_detect_nonexistent_binary() {
        let info = detect_singbox_info(Some(Path::new("/nonexistent/sing-box-mock")));
        assert!(!info.installed);
        assert_eq!(info.size_bytes, 0);
        assert!(info.variant.is_none());
    }
}
