//! Детект соседних инструментов (DPI / PBR / клиентские прокси-стеки).
//!
//! OpenStream — прозрачный MITM (nft divert) перед egress к CDN.
//! Соседи обеспечивают доступ; strip — только если divert поймал HLS.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neighbor {
    pub name: String,
    pub detected: bool,
    pub hint: String,
}

struct Candidate {
    name: &'static str,
    hint: &'static str,
    /// Имена для /proc/comm, /usr/bin, init.d, UCI.
    markers: &'static [&'static str],
    /// Доп. пути (ядра SSClash, меню LuCI и т.п.).
    extra_paths: &'static [&'static str],
    /// При детектe → mode_hint prefer_transparent_check_tpws (конфликт с divert).
    prefer_explicit: bool,
}

const CANDIDATES: &[Candidate] = &[
    // --- DPI bypass (пакетный / SOCKS) ---
    Candidate {
        name: "nfqws",
        hint: "zapret nfqws",
        markers: &["nfqws"],
        extra_paths: &[],
        prefer_explicit: false,
    },
    Candidate {
        name: "nfqws2",
        hint: "zapret2 nfqws2",
        markers: &["nfqws2"],
        extra_paths: &[],
        prefer_explicit: false,
    },
    Candidate {
        name: "tpws",
        hint: "zapret tpws transparent — may conflict with openstream divert",
        markers: &["tpws"],
        extra_paths: &[],
        prefer_explicit: true,
    },
    Candidate {
        name: "zapret",
        hint: "zapret service",
        markers: &["zapret"],
        extra_paths: &[],
        prefer_explicit: false,
    },
    Candidate {
        name: "byedpi",
        hint: "ByeDPI (ciadpi SOCKS / DPI) — OK on OSE egress",
        markers: &["byedpi", "ciadpi"],
        extra_paths: &[],
        prefer_explicit: false,
    },
    // --- PBR / sing-box обвязки (podkop family) — transparent OK if divert before TUN ---
    Candidate {
        name: "podkop",
        hint: "podkop / sing-box routing",
        markers: &["podkop"],
        extra_paths: &[],
        prefer_explicit: false,
    },
    Candidate {
        name: "netshift",
        hint: "NetShift (podkop fork) / sing-box",
        markers: &["netshift"],
        extra_paths: &[],
        prefer_explicit: false,
    },
    Candidate {
        name: "forkop",
        hint: "Forkop / Podkop Plus (podkop fork)",
        markers: &["forkop"],
        extra_paths: &[],
        prefer_explicit: false,
    },
    // --- Clash / Mihomo ---
    Candidate {
        name: "ssclash",
        hint: "SSClash (Mihomo /opt/clash)",
        markers: &["ssclash"],
        extra_paths: &[
            "/opt/clash/bin/clash",
            "/opt/clash/config.yaml",
            "/usr/share/luci/menu.d/luci-app-ssclash.json",
        ],
        prefer_explicit: false,
    },
    Candidate {
        name: "openclash",
        hint: "OpenClash",
        markers: &["openclash"],
        extra_paths: &[
            "/etc/openclash",
            "/usr/share/luci/menu.d/luci-app-openclash.json",
        ],
        prefer_explicit: false,
    },
    Candidate {
        name: "mihomo",
        hint: "mihomo / Meta kernel",
        markers: &["mihomo"],
        extra_paths: &["/usr/bin/mihomo"],
        prefer_explicit: false,
    },
    Candidate {
        name: "clash",
        hint: "clash / clash-meta process",
        markers: &["clash", "clash-meta"],
        extra_paths: &[],
        prefer_explicit: false,
    },
    Candidate {
        name: "passwall",
        hint: "PassWall",
        markers: &["passwall"],
        extra_paths: &[
            "/etc/config/passwall",
            "/usr/share/luci/menu.d/luci-app-passwall.json",
        ],
        prefer_explicit: false,
    },
    Candidate {
        name: "passwall2",
        hint: "PassWall 2",
        markers: &["passwall2"],
        extra_paths: &[
            "/etc/config/passwall2",
            "/usr/share/luci/menu.d/luci-app-passwall2.json",
        ],
        prefer_explicit: false,
    },
    Candidate {
        name: "homeproxy",
        hint: "HomeProxy (sing-box)",
        markers: &["homeproxy"],
        extra_paths: &["/etc/config/homeproxy"],
        prefer_explicit: false,
    },
    Candidate {
        name: "sing-box",
        hint: "sing-box",
        markers: &["sing-box", "singbox"],
        extra_paths: &[],
        prefer_explicit: false,
    },
    Candidate {
        name: "xray",
        hint: "xray",
        markers: &["xray"],
        extra_paths: &[],
        prefer_explicit: false,
    },
    Candidate {
        name: "hydraroute",
        hint: "HydraRoute",
        markers: &["hydraroute"],
        extra_paths: &[],
        prefer_explicit: false,
    },
    Candidate {
        name: "redsocks",
        hint: "redsocks / transparent redirect to SOCKS — may conflict",
        markers: &["redsocks", "redsocks2"],
        extra_paths: &[],
        prefer_explicit: true,
    },
    Candidate {
        name: "tun2proxy",
        hint: "tun2proxy",
        markers: &["tun2proxy"],
        extra_paths: &[],
        prefer_explicit: false,
    },
    Candidate {
        name: "hev-socks5-tunnel",
        hint: "HevSocks5Tunnel",
        markers: &["hev-socks5-tunnel", "hevsocks5tunnel"],
        extra_paths: &[],
        prefer_explicit: false,
    },
];

/// Проверка наличия процессов/бинарников/init/UCI на типичных путях OpenWrt.
pub fn detect_neighbors() -> Vec<Neighbor> {
    CANDIDATES
        .iter()
        .map(|c| Neighbor {
            name: c.name.to_string(),
            detected: is_present(c),
            hint: c.hint.to_string(),
        })
        .collect()
}

fn is_present(c: &Candidate) -> bool {
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for ent in entries.flatten() {
            let pid_path = ent.path().join("comm");
            if let Ok(comm) = std::fs::read_to_string(&pid_path) {
                let proc_name = comm.trim();
                for m in c.markers {
                    if proc_name == *m || proc_name.contains(m) {
                        return true;
                    }
                }
            }
        }
    }

    for m in c.markers {
        let paths = [
            format!("/usr/bin/{m}"),
            format!("/usr/sbin/{m}"),
            format!("/opt/bin/{m}"),
            format!("/bin/{m}"),
            format!("/etc/init.d/{m}"),
            format!("/etc/config/{m}"),
        ];
        if paths.iter().any(|p| Path::new(p).exists()) {
            return true;
        }
    }

    c.extra_paths.iter().any(|p| Path::new(p).exists())
}

pub fn coexistence_ok(neighbors: &[Neighbor]) -> bool {
    !neighbors.iter().any(|n| {
        n.detected
            && CANDIDATES
                .iter()
                .any(|c| c.name == n.name && c.prefer_explicit)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(name: &str, detected: bool) -> Neighbor {
        Neighbor {
            name: name.into(),
            detected,
            hint: String::new(),
        }
    }

    #[test]
    fn coexistence_ok_with_nfqws_and_podkop() {
        assert!(coexistence_ok(&[n("nfqws", true), n("podkop", true)]));
        assert!(coexistence_ok(&[n("sing-box", true)]));
        assert!(coexistence_ok(&[n("ssclash", true)]));
        assert!(coexistence_ok(&[n("byedpi", true)]));
    }

    #[test]
    fn coexistence_warns_on_conflicting_transparent() {
        assert!(!coexistence_ok(&[n("tpws", true)]));
        assert!(!coexistence_ok(&[n("redsocks", true)]));
    }

    #[test]
    fn candidates_include_ssclash_and_byedpi() {
        let names: Vec<_> = CANDIDATES.iter().map(|c| c.name).collect();
        assert!(names.contains(&"ssclash"));
        assert!(names.contains(&"byedpi"));
        assert!(names.contains(&"netshift"));
        assert!(names.contains(&"forkop"));
    }
}
