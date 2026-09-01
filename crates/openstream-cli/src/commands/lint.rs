use std::fs;
use std::path::{Path, PathBuf};
use colored::Colorize;
use openstream_rule::load_rule_file;

pub fn execute_lint(paths: &[PathBuf]) -> Result<(), anyhow::Error> {
    let mut files_to_check = Vec::new();

    for p in paths {
        if p.is_dir() {
            collect_rule_files(p, &mut files_to_check)?;
        } else if p.is_file() {
            files_to_check.push(p.clone());
        } else {
            eprintln!("{} Path does not exist: {}", "ERROR:".red().bold(), p.display());
        }
    }

    if files_to_check.is_empty() {
        println!("{}", "No .osrule.yaml files found to lint.".yellow());
        return Ok(());
    }

    println!("{}\n", format!("🔍 Linting {} OpenStream 2.0 rule(s)...", files_to_check.len()).bold());

    let mut passed = 0;
    let mut failed = 0;

    for file in &files_to_check {
        match load_rule_file(file) {
            Ok(manifest) => {
                println!(
                    "  {} {} {} [{}] (matches: {})",
                    "✓".green().bold(),
                    file.display().to_string().bold(),
                    format!("v{}", manifest.version).cyan(),
                    manifest.id.dimmed(),
                    manifest.matches.len()
                );
                passed += 1;
            }
            Err(e) => {
                println!(
                    "  {} {}\n    {}",
                    "✗".red().bold(),
                    file.display().to_string().bold(),
                    format!("Error: {}", e).red()
                );
                failed += 1;
            }
        }
    }

    println!();
    if failed == 0 {
        println!("{}", format!("✨ All {} rule(s) passed validation!", passed).green().bold());
        Ok(())
    } else {
        anyhow::bail!("Lint failed: {} passed, {} failed", passed, failed);
    }
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
