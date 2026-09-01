//
//  zapret2.rs
//  openstream-backend-openwrt
//
//  Интеграция с пакетом 1andrevich/zapret2-openwrt (nfqws2):
//  - Обнаружение бинарника /usr/bin/nfqws2
//  - Стандартные оптимизированные пресеты (YouTube 4K, Discord Voice, General)
//  - Генерация аргументов запуска демона и привязка к NFQUEUE очереди 1088
//

use std::path::Path;

pub const DEFAULT_ZAPRET2_QUEUE_NUM: u16 = 1088;
pub const ZAPRET2_BIN_PATH: &str = "/usr/bin/nfqws2";
pub const ZAPRET2_LEGACY_BIN_PATH: &str = "/usr/bin/nfqws";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Zapret2Preset {
    pub name: &'static str,
    pub description: &'static str,
    pub tcp_args: &'static str,
    pub udp_args: Option<&'static str>,
}

pub const BUILTIN_PRESETS: &[Zapret2Preset] = &[
    Zapret2Preset {
        name: "youtube_4k",
        description: "YouTube & Googlevideo: split2 по 2-му байту с badseq",
        tcp_args: "--dpi-desync=split2 --dpi-desync-split-pos=2 --dpi-desync-fooling=badseq",
        udp_args: Some("--filter-udp=443 --dpi-desync=fake --dpi-desync-repeats=2"),
    },
    Zapret2Preset {
        name: "discord_voice",
        description: "Discord: обход блокировок голосовых шлюзов RTC и WebRTC",
        tcp_args: "--dpi-desync=split2 --dpi-desync-split-pos=1",
        udp_args: Some("--filter-udp=50000:65535 --dpi-desync=fake --dpi-desync-any-protocol"),
    },
    Zapret2Preset {
        name: "general_multisplit",
        description: "Универсальный обход: multisplit по SNI и смещению midsld",
        tcp_args: "--dpi-desync=multisplit --dpi-desync-split-pos=1,midsld",
        udp_args: None,
    },
];

/// Проверка наличия установленного zapret2 в системе
pub fn is_zapret2_installed() -> bool {
    Path::new(ZAPRET2_BIN_PATH).exists() || Path::new(ZAPRET2_LEGACY_BIN_PATH).exists()
}

/// Получение аргументов для запуска nfqws2 по имени пресета или кастомной строке
pub fn resolve_zapret2_args(
    preset_name: &str,
    custom_args: Option<&str>,
    queue_num: u16,
) -> String {
    if let Some(custom) = custom_args {
        if !custom.trim().is_empty() {
            return format!("--qnum={} {}", queue_num, custom.trim());
        }
    }

    for preset in BUILTIN_PRESETS {
        if preset.name == preset_name {
            let mut args = format!("--qnum={} {}", queue_num, preset.tcp_args);
            if let Some(udp) = preset.udp_args {
                args.push(' ');
                args.push_str(udp);
            }
            return args;
        }
    }

    // Fallback на general_multisplit
    format!("--qnum={} {}", queue_num, BUILTIN_PRESETS[2].tcp_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_zapret2_builtin_preset() {
        let args = resolve_zapret2_args("youtube_4k", None, 1088);
        assert!(args.contains("--qnum=1088"));
        assert!(args.contains("--dpi-desync=split2"));
    }

    #[test]
    fn test_resolve_zapret2_custom_args() {
        let args = resolve_zapret2_args("custom", Some("--dpi-desync=fake --dpi-desync-ttl=4"), 1088);
        assert_eq!(args, "--qnum=1088 --dpi-desync=fake --dpi-desync-ttl=4");
    }
}
