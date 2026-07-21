//! Универсальный rule engine: host patterns + detector rules из YAML.

use std::fs;
use std::path::Path;

use ose_detector::{Rule, RuleKind};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RulesError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Подстроки или суффиксы хоста (`ttvnw.net`, `kick.com`).
    pub hosts: Vec<String>,
    #[serde(default)]
    pub rules: Vec<RuleDef>,
    #[serde(default = "default_true")]
    pub strip_prefetch_on_ads: bool,
    #[serde(default = "default_true")]
    pub rewrite_master: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDef {
    pub kind: RuleKind,
    #[serde(default)]
    pub pattern: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl RuleDef {
    pub fn into_rule(self) -> Rule {
        Rule {
            kind: self.kind,
            pattern: self.pattern,
            enabled: self.enabled,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RulesFile {
    #[serde(default)]
    pub rulesets: Vec<RuleSet>,
}

impl RulesFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, RulesError> {
        let text = fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&text)?)
    }

    pub fn from_yaml(text: &str) -> Result<Self, RulesError> {
        Ok(serde_yaml::from_str(text)?)
    }
}

/// Проверка хоста: case-insensitive contains любого паттерна.
pub fn host_matches(host: &str, patterns: &[String]) -> bool {
    let h = host.to_ascii_lowercase();
    patterns.iter().any(|p| {
        let p = p.trim().trim_start_matches('*').trim_start_matches('.');
        let p = p.to_ascii_lowercase();
        !p.is_empty() && h.contains(&p)
    })
}

pub fn kick_default() -> RuleSet {
    RuleSet {
        name: "kick".into(),
        enabled: true,
        hosts: vec![
            "kick.com".into(),
            "kick.video".into(),
            "kickusercontent".into(),
        ],
        rules: vec![
            RuleDef {
                kind: RuleKind::Contains,
                pattern: "stitched".into(),
                enabled: true,
            },
            RuleDef {
                kind: RuleKind::DateRange,
                pattern: "ad".into(),
                enabled: true,
            },
            RuleDef {
                kind: RuleKind::ExtInfNotLive,
                pattern: "live".into(),
                enabled: false, // Kick может не использовать ,live
            },
        ],
        strip_prefetch_on_ads: true,
        rewrite_master: true,
    }
}

pub fn trovo_default() -> RuleSet {
    RuleSet {
        name: "trovo".into(),
        enabled: true,
        hosts: vec!["trovo.live".into(), "trovo.com".into()],
        rules: vec![
            RuleDef {
                kind: RuleKind::Contains,
                pattern: "stitched".into(),
                enabled: true,
            },
            RuleDef {
                kind: RuleKind::Contains,
                pattern: "advertisement".into(),
                enabled: true,
            },
            RuleDef {
                kind: RuleKind::DateRange,
                pattern: "ad".into(),
                enabled: true,
            },
        ],
        strip_prefetch_on_ads: true,
        rewrite_master: true,
    }
}

/// YouTube Live HLS (эвристики; полевая калибровка отдельно).
pub fn youtube_default() -> RuleSet {
    RuleSet {
        name: "youtube".into(),
        enabled: true,
        hosts: vec![
            "googlevideo.com".into(),
            "youtube.com".into(),
            "ytimg.com".into(),
            "ggpht.com".into(),
        ],
        rules: vec![
            RuleDef {
                kind: RuleKind::Contains,
                pattern: "oad=".into(), // occasionally seen on ad variants
                enabled: false,
            },
            RuleDef {
                kind: RuleKind::Contains,
                pattern: "/ad_".into(),
                enabled: true,
            },
            RuleDef {
                kind: RuleKind::Contains,
                pattern: "advertisement".into(),
                enabled: true,
            },
            RuleDef {
                kind: RuleKind::DateRange,
                pattern: "ad".into(),
                enabled: false,
            },
        ],
        strip_prefetch_on_ads: true,
        rewrite_master: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_match() {
        assert!(host_matches(
            "video.kick.com",
            &["kick.com".into()]
        ));
        assert!(!host_matches("example.com", &["kick.com".into()]));
    }

    #[test]
    fn parse_rules_yaml() {
        let y = r#"
rulesets:
  - name: demo
    hosts: ["example.com"]
    rules:
      - kind: contains
        pattern: stitched
"#;
        let f = RulesFile::from_yaml(y).unwrap();
        assert_eq!(f.rulesets.len(), 1);
        assert_eq!(f.rulesets[0].name, "demo");
    }
}
