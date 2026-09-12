pub mod adapter;
pub mod dnsmasq;
pub mod monitor;
pub mod nftables;
pub mod singbox;
pub mod zapret2;

pub use adapter::OpenWrtBackend;
pub use dnsmasq::{generate_dnsmasq_config, sanitize_domain};
pub use monitor::{FlowRecord, FlowTracker, RoutingSection};
pub use nftables::{generate_nftables_rules, generate_nftables_sets};
pub use singbox::{detect_singbox_info, generate_singbox_inbound_config, SingBoxInfo, SingBoxVariant};
pub use zapret2::{is_zapret2_installed, resolve_zapret2_args, BUILTIN_PRESETS};

use std::fs;
use std::path::Path;
use openstream_core::PolicyEngine;
use openstream_rule::load_rule_file;

/// Загрузка всех правил .osrule.yaml из каталога, компиляция и экспорт конфигураций для dnsmasq и nftables
pub fn compile_rules_dir(
    rules_dir: impl AsRef<Path>,
    dnsmasq_out: impl AsRef<Path>,
    nft_out: impl AsRef<Path>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let engine = PolicyEngine::new();
    let mut loaded_count = 0;

    let dir = rules_dir.as_ref();
    if dir.exists() && dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                // Рекурсивный обход поддиректорий (streaming, privacy и т.д.)
                for sub_entry in fs::read_dir(&path)? {
                    let sub_entry = sub_entry?;
                    let sub_path = sub_entry.path();
                    if sub_path.is_file() && sub_path.to_string_lossy().ends_with(".osrule.yaml") {
                        match load_rule_file(&sub_path) {
                            Ok(rule) => {
                                engine.add_rule(rule);
                                loaded_count += 1;
                            }
                            Err(err) => {
                                eprintln!(
                                    "[openstream] Внимание: не удалось загрузить правило '{}': {}",
                                    sub_path.display(),
                                    err
                                );
                            }
                        }
                    }
                }
            } else if path.is_file() && path.to_string_lossy().ends_with(".osrule.yaml") {
                match load_rule_file(&path) {
                    Ok(rule) => {
                        engine.add_rule(rule);
                        loaded_count += 1;
                    }
                    Err(err) => {
                        eprintln!(
                            "[openstream] Внимание: не удалось загрузить правило '{}': {}",
                            path.display(),
                            err
                        );
                    }
                }
            }
        }
    }

    let compiled = engine.compile_for_backend();
    let dnsmasq_cfg = generate_dnsmasq_config(&compiled, "inet", "openstream");
    let nft_cfg = generate_nftables_sets(&compiled, "inet", "openstream");

    if let Some(parent) = dnsmasq_out.as_ref().parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(dnsmasq_out, dnsmasq_cfg)?;

    if let Some(parent) = nft_out.as_ref().parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(nft_out, nft_cfg)?;

    Ok(loaded_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_compile_reference_rules() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let rules_dir = manifest_dir.parent().unwrap().parent().unwrap().join("rules");

        let temp_dir = std::env::temp_dir().join("openstream_test_compile");
        let dnsmasq_out = temp_dir.join("dnsmasq.conf");
        let nft_out = temp_dir.join("openstream.nft");

        let count = compile_rules_dir(&rules_dir, &dnsmasq_out, &nft_out)
            .expect("Компиляция правил каталога rules должна пройти успешно");

        assert!(count >= 3, "Должно быть загружено не менее 3 эталонных правил");
        assert!(dnsmasq_out.exists());
        assert!(nft_out.exists());

        let dnsmasq_content = fs::read_to_string(&dnsmasq_out).unwrap();
        assert!(dnsmasq_content.contains("address=/edge.ads.twitch.tv/0.0.0.0"));
        assert!(dnsmasq_content.contains("nftset=/googlevideo.com/4#inet#openstream#dpi_evasive"));

        let nft_content = fs::read_to_string(&nft_out).unwrap();
        assert!(nft_content.contains("table inet openstream {"));
        assert!(nft_content.contains("set dpi_evasive {"));

        let _ = fs::remove_dir_all(temp_dir);
    }
}
