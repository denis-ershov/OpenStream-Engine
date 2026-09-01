pub mod crypto;
pub mod schema;
pub mod validator;

use std::fs;
use std::path::Path;
use thiserror::Error;

pub use crypto::{sign_bytes, verify_signature, CryptoError};
pub use schema::{MatchRule, RoutingAction, RuleManifest, StrategyDef, CURRENT_SCHEMA_VERSION};
pub use validator::{validate_manifest, ValidationError};

#[derive(Debug, Error)]
pub enum RuleError {
    #[error("Ошибка ввода-вывода при чтении файла правила: {0}")]
    Io(#[from] std::io::Error),
    #[error("Ошибка парсинга YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Ошибка валидации правила: {0}")]
    Validation(#[from] ValidationError),
    #[error("Криптографическая ошибка: {0}")]
    Crypto(#[from] CryptoError),
}

/// Загрузка и валидация манифеста правила из YAML-строки
pub fn parse_rule_yaml(yaml_str: &str) -> Result<RuleManifest, RuleError> {
    let manifest: RuleManifest = serde_yaml::from_str(yaml_str)?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

/// Загрузка и валидация манифеста правила из файла
pub fn load_rule_file(path: impl AsRef<Path>) -> Result<RuleManifest, RuleError> {
    let content = fs::read_to_string(path)?;
    parse_rule_yaml(&content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    const VALID_TWITCH_YAML: &str = r#"
schema_version: "2.0"
id: "org.openstream.rules.twitch"
name: "Twitch Optimizer"
version: "1.0.0"
capabilities_required: ["domain_routing"]
matches:
  - group: "token"
    domains: ["gql.twitch.tv"]
    strategy: "adfree"
  - group: "video"
    domains: ["*.live-video.net", "ttvnw.net"]
    action: "direct"
strategies:
  adfree:
    preference: ["geo:al", "direct"]
"#;

    #[test]
    fn test_valid_manifest_parse() {
        let manifest = parse_rule_yaml(VALID_TWITCH_YAML).expect("Должен успешно распарсить");
        assert_eq!(manifest.id, "org.openstream.rules.twitch");
        assert_eq!(manifest.matches.len(), 2);
        assert_eq!(manifest.strategies.len(), 1);
    }

    #[test]
    fn test_invalid_schema_version() {
        let invalid = r#"
schema_version: "1.0"
id: "test"
name: "test"
version: "1.0.0"
"#;
        let res = parse_rule_yaml(invalid);
        assert!(res.is_err());
        match res.unwrap_err() {
            RuleError::Validation(ValidationError::UnsupportedSchema(v, _)) => {
                assert_eq!(v, "1.0");
            }
            other => panic!("Ожидалась UnsupportedSchema, получено: {:?}", other),
        }
    }

    #[test]
    fn test_undefined_strategy() {
        let invalid = r#"
schema_version: "2.0"
id: "test"
name: "test"
version: "1.0.0"
matches:
  - domains: ["example.com"]
    strategy: "non_existent"
"#;
        let res = parse_rule_yaml(invalid);
        assert!(matches!(
            res.unwrap_err(),
            RuleError::Validation(ValidationError::UndefinedStrategy(_))
        ));
    }

    #[test]
    fn test_secops_blocks_apple_without_flag() {
        let malicious = r#"
schema_version: "2.0"
id: "hijack_apple"
name: "Apple Hijack"
version: "1.0.0"
matches:
  - domains: ["gateway.icloud.com"]
    action: "direct"
"#;
        let res = parse_rule_yaml(malicious);
        assert!(matches!(
            res.unwrap_err(),
            RuleError::Validation(ValidationError::SensitiveDomainBlocked(_))
        ));
    }

    #[test]
    fn test_secops_allows_sensitive_with_explicit_flag() {
        let allowed = r#"
schema_version: "2.0"
id: "allow_apple"
name: "Apple Allow"
version: "1.0.0"
allow_sensitive: true
matches:
  - domains: ["gateway.icloud.com"]
    action: "direct"
"#;
        assert!(parse_rule_yaml(allowed).is_ok());
    }

    #[test]
    fn test_crypto_signing_and_verification() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        let data = VALID_TWITCH_YAML.as_bytes();
        let sig = sign_bytes(data, &signing_key);

        assert!(verify_signature(data, &sig, verifying_key.as_bytes()).is_ok());

        // Проверка с измененными данными
        let corrupted_data = b"schema_version: 2.0 Corrupted";
        assert!(verify_signature(corrupted_data, &sig, verifying_key.as_bytes()).is_err());
    }

    #[test]
    fn test_schema_2_1_and_shadowing_detection() {
        let yaml_2_1 = r#"
schema_version: "2.1"
id: "org.openstream.rules.shadow_test"
name: "Shadowing Test"
version: "1.0.0"
matches:
  - domains: ["*.youtube.com"]
    action: "direct"
  - domains: ["music.youtube.com"]
    action:
      zapret2:
        preset: "youtube_4k"
"#;
        let manifest = parse_rule_yaml(yaml_2_1).expect("Должен распарсить Schema 2.1");
        assert_eq!(manifest.schema_version, "2.1");

        let warnings = validator::detect_shadowing(&manifest);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].broader_pattern, "*.youtube.com");
        assert_eq!(warnings[0].shadowed_pattern, "music.youtube.com");
    }
}

