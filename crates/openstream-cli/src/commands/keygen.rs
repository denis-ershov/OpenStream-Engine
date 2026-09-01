use colored::Colorize;
use ed25519_dalek::SigningKey;
use rand_core::OsRng;
use std::fs;
use std::path::PathBuf;

pub fn execute_keygen(out_prefix: Option<PathBuf>) -> Result<(), anyhow::Error> {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    let priv_hex = hex::encode(signing_key.to_bytes());
    let pub_hex = hex::encode(verifying_key.to_bytes());

    if let Some(prefix) = out_prefix {
        let priv_path = prefix.with_extension("key");
        let pub_path = prefix.with_extension("pub");

        fs::write(&priv_path, &priv_hex)?;
        fs::write(&pub_path, &pub_hex)?;

        println!("{}", "🔑 Generated Ed25519 Keypair:".green().bold());
        println!("  Private Key: {}", priv_path.display().to_string().bold());
        println!("  Public Key:  {}", pub_path.display().to_string().bold());
    } else {
        println!("{}", "🔑 Generated Ed25519 Keypair:".green().bold());
        println!("  Private Key (Keep secret!): {}", priv_hex.yellow().bold());
        println!("  Public Key:                {}", pub_hex.cyan().bold());
    }

    Ok(())
}
