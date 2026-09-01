use std::fs;
use std::path::PathBuf;
use colored::Colorize;
use openstream_rule::sign_bytes;

pub fn execute_sign(file: PathBuf, key: String, out_sig: Option<PathBuf>) -> Result<(), anyhow::Error> {
    let content = fs::read(&file)?;

    // Чтение ключа: либо путь к файлу, либо hex-строка
    let key_hex = if let Ok(file_content) = fs::read_to_string(&key) {
        file_content.trim().to_string()
    } else {
        key.trim().to_string()
    };

    let key_vec = hex::decode(&key_hex)
        .map_err(|e| anyhow::anyhow!("Invalid hex private key: {}", e))?;

    let key_bytes: [u8; 32] = key_vec
        .try_into()
        .map_err(|_| anyhow::anyhow!("Ed25519 private key must be exactly 32 bytes"))?;

    let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_bytes);
    let sig_bytes = sign_bytes(&content, &signing_key);
    let sig_hex = hex::encode(sig_bytes);

    let sig_dest = out_sig.unwrap_or_else(|| file.with_extension("osrule.sig"));
    fs::write(&sig_dest, &sig_hex)?;

    println!("{}", "✍️ Successfully signed rule manifest:".green().bold());
    println!("  File:      {}", file.display());
    println!("  Signature: {}", sig_dest.display().to_string().cyan().bold());
    println!("  Sig Hex:   {}", sig_hex.dimmed());

    Ok(())
}
