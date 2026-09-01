use std::fs;
use std::path::{Path, PathBuf};
use colored::Colorize;
use openstream_rule::load_rule_file;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug)]
pub struct CatalogRuleEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub capabilities: Vec<String>,
    pub matches_count: usize,
    pub sha256: String,
    pub signature: Option<String>,
    pub relative_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommunityRulesCatalog {
    pub catalog_version: String,
    pub generated_at: String,
    pub total_rules: usize,
    pub rules: Vec<CatalogRuleEntry>,
}

pub fn execute_index(rules_dir: PathBuf, out_path: PathBuf) -> Result<(), anyhow::Error> {
    if !rules_dir.exists() || !rules_dir.is_dir() {
        anyhow::bail!("Rules directory not found: {}", rules_dir.display());
    }

    let mut rule_files = Vec::new();
    collect_rule_files(&rules_dir, &mut rule_files)?;

    println!(
        "{}",
        format!("📦 Generating Community Index for {} rule(s)...", rule_files.len()).bold()
    );

    let mut catalog_entries = Vec::new();

    for file in rule_files {
        let manifest = match load_rule_file(&file) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("  {} Skipping invalid rule {}: {}", "⚠".yellow().bold(), file.display(), e);
                continue;
            }
        };

        let file_bytes = fs::read(&file)?;
        let mut hasher = Sha256::new();
        hasher.update(&file_bytes);
        let sha256_hex = hex::encode(hasher.finalize());

        let sig_path = file.with_extension("osrule.sig");
        let sig_opt = if sig_path.exists() {
            fs::read_to_string(&sig_path).ok().map(|s| s.trim().to_string())
        } else {
            None
        };

        let rel_path = file
            .strip_prefix(&rules_dir)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");

        let caps = manifest
            .capabilities_required
            .iter()
            .map(|c| c.to_string())
            .collect();

        catalog_entries.push(CatalogRuleEntry {
            id: manifest.id,
            name: manifest.name,
            version: manifest.version,
            author: manifest.author,
            description: manifest.description,
            capabilities: caps,
            matches_count: manifest.matches.len(),
            sha256: sha256_hex,
            signature: sig_opt,
            relative_path: rel_path,
        });
    }

    let catalog = CommunityRulesCatalog {
        catalog_version: "2.0".to_string(),
        generated_at: "2026-09-01T17:00:00Z".to_string(),
        total_rules: catalog_entries.len(),
        rules: catalog_entries,
    };

    let json_bytes = serde_json::to_string_pretty(&catalog)?;
    if let Some(parent) = out_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(&out_path, json_bytes)?;

    println!(
        "{} Saved catalog with {} entries to {}",
        "✓".green().bold(),
        catalog.total_rules,
        out_path.display().to_string().cyan().bold()
    );

    Ok(())
}

fn collect_rule_files(dir: &Path, acc: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_rule_files(&path, acc)?;
            } else if path.is_file() && path.to_string_lossy().ends_with(".osrule.yaml") {
                acc.push(path);
            }
        }
    }
    Ok(())
}
