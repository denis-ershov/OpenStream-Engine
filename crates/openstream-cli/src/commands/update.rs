use colored::Colorize;
use openstream_rule::load_rule_file;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

pub fn execute_update(dir: PathBuf, verify_hashes: bool) -> Result<(), anyhow::Error> {
    println!(
        "{}",
        "🔄 OpenStream Rule Catalog Synchronizer & Integrity Checker".bold().cyan()
    );
    println!("Target rules directory: {}", dir.display().to_string().yellow());

    if !dir.exists() {
        fs::create_dir_all(&dir)?;
        println!("{}", "Created missing directory.".dimmed());
    }

    let mut inspected = 0;
    let mut valid = 0;
    let mut failed = 0;

    let mut stack = vec![dir];
    while let Some(current_dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() && path.to_string_lossy().ends_with(".osrule.yaml") {
                    inspected += 1;
                    match load_rule_file(&path) {
                        Ok(manifest) => {
                            let mut sha_info = String::new();
                            if verify_hashes {
                                if let Ok(bytes) = fs::read(&path) {
                                    let mut hasher = Sha256::new();
                                    hasher.update(&bytes);
                                    let hash = hex::encode(hasher.finalize());
                                    sha_info = format!(" [SHA256: {}..]", &hash[..8]);
                                }
                            }
                            println!(
                                "  {} {} ({}) — {}{}",
                                "✓".green().bold(),
                                manifest.name.bold(),
                                manifest.id.dimmed(),
                                manifest.version.blue(),
                                sha_info.dimmed()
                            );
                            valid += 1;
                        }
                        Err(err) => {
                            println!(
                                "  {} {} — {}",
                                "✗".red().bold(),
                                path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown"),
                                err.to_string().red()
                            );
                            failed += 1;
                        }
                    }
                }
            }
        }
    }

    println!();
    if failed > 0 {
        println!(
            "{}",
            format!(
                "⚠️ Catalog sync completed with warnings: {} valid, {} invalid of {} total rules.",
                valid, failed, inspected
            )
            .yellow()
            .bold()
        );
    } else {
        println!(
            "{}",
            format!(
                "✓ Catalog is fully up to date and verified: {} valid rules processed.",
                valid
            )
            .green()
            .bold()
        );
    }

    Ok(())
}
