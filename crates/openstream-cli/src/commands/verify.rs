use std::fs;
use std::path::PathBuf;
use colored::Colorize;
use openstream_rule::verify_signature;

pub fn execute_verify(file: PathBuf, pubkey: String, sig: Option<PathBuf>) -> Result<(), anyhow::Error> {
    let content = fs::read(&file)?;

    let pubkey_hex = if let Ok(file_content) = fs::read_to_string(&pubkey) {
        file_content.trim().to_string()
    } else {
        pubkey.trim().to_string()
    };

    let pubkey_vec = hex::decode(&pubkey_hex)
        .map_err(|e| anyhow::anyhow!("Invalid hex public key: {}", e))?;

    let pubkey_bytes: [u8; 32] = pubkey_vec
        .try_into()
        .map_err(|_| anyhow::anyhow!("Ed25519 public key must be exactly 32 bytes"))?;

    let sig_path = sig.unwrap_or_else(|| file.with_extension("osrule.sig"));
    if !sig_path.exists() {
        anyhow::bail!("Signature file not found: {}", sig_path.display());
    }

    let sig_hex = fs::read_to_string(&sig_path)?;
    let sig_vec = hex::decode(sig_hex.trim())
        .map_err(|e| anyhow::anyhow!("Invalid signature hex: {}", e))?;

    let sig_bytes: [u8; 64] = sig_vec
        .try_into()
        .map_err(|_| anyhow::anyhow!("Ed25519 signature must be exactly 64 bytes"))?;

    match verify_signature(&content, &sig_bytes, &pubkey_bytes) {
        Ok(()) => {
            println!(
                "{} Signature is {} for {}",
                "✓".green().bold(),
                "VALID".green().bold(),
                file.display()
            );
            Ok(())
        }
        Err(e) => {
            println!(
                "{} Signature is {} for {}: {}",
                "✗".red().bold(),
                "INVALID / TAMPERED".red().bold(),
                file.display(),
                e
            );
            anyhow::bail!("Cryptographic verification failed for {}", file.display());
        }
    }
}
