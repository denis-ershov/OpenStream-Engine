//! Детект соседних инструментов (zapret/podkop/…).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neighbor {
    pub name: String,
    pub detected: bool,
    pub hint: String,
}

const CANDIDATES: &[(&str, &str)] = &[
    ("nfqws", "zapret nfqws"),
    ("nfqws2", "zapret2 nfqws2"),
    ("tpws", "zapret tpws transparent"),
    ("zapret", "zapret service"),
    ("podkop", "podkop / sing-box routing"),
    ("sing-box", "sing-box"),
    ("xray", "xray"),
    ("hydraroute", "HydraRoute"),
];

/// Проверка наличия процессов/бинарников в PATH и типичных путях OpenWrt.
pub fn detect_neighbors() -> Vec<Neighbor> {
    CANDIDATES
        .iter()
        .map(|(name, hint)| Neighbor {
            name: (*name).to_string(),
            detected: is_present(name),
            hint: (*hint).to_string(),
        })
        .collect()
}

fn is_present(name: &str) -> bool {
    // Процессы (Linux/OpenWrt).
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for ent in entries.flatten() {
            let pid_path = ent.path().join("comm");
            if let Ok(comm) = std::fs::read_to_string(&pid_path) {
                if comm.trim() == name || comm.trim().contains(name) {
                    return true;
                }
            }
        }
    }
    // Бинарник в типичных путях.
    let paths = [
        format!("/usr/bin/{name}"),
        format!("/usr/sbin/{name}"),
        format!("/opt/bin/{name}"),
        format!("/bin/{name}"),
    ];
    paths.iter().any(|p| std::path::Path::new(p).exists())
}

pub fn coexistence_ok(neighbors: &[Neighbor]) -> bool {
    // Предупреждение если tpws detected — transparent может конфликтовать с redirect_whitelist.
    !neighbors
        .iter()
        .any(|n| n.detected && (n.name == "tpws" || n.name == "podkop"))
}
