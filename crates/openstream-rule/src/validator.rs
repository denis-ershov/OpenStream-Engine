use thiserror::Error;
use crate::schema::RuleManifest;

#[derive(Debug, Error, PartialEq)]
pub enum ValidationError {
    #[error("Недопустимая версия схемы: '{0}', поддерживается '{1}'")]
    UnsupportedSchema(String, &'static str),

    #[error("Поле '{0}' не может быть пустым")]
    EmptyField(&'static str),

    #[error("Некорректный синтаксис домена: '{0}' ({1})")]
    InvalidDomain(String, &'static str),

    #[error("SecOps: Перехват защищенного домена '{0}' запрещен без явного флага 'allow_sensitive: true'")]
    SensitiveDomainBlocked(String),

    #[error("Стратегия '{0}' указана в match, но не определена в блоке strategies")]
    UndefinedStrategy(String),

    #[error("Стратегия '{0}' имеет пустой список предпочтений (preference)")]
    EmptyStrategy(String),
}

/// Список системных критических доменов, перехват которых заблокирован по умолчанию
const PROTECTED_SYSTEM_DOMAINS: &[&str] = &[
    "apple.com",
    "icloud.com",
    "aaplimg.com",
    "microsoft.com",
    "windowsupdate.com",
    "live.com",
    "gosuslugi.ru",
    "sberbank.ru",
    "tbank.ru",
    "vtb.ru",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowingWarning {
    pub broader_rule_index: usize,
    pub shadowed_rule_index: usize,
    pub broader_pattern: String,
    pub shadowed_pattern: String,
    pub message: String,
}

/// Валидация манифеста правила перед компиляцией в ядро
pub fn validate_manifest(manifest: &RuleManifest) -> Result<(), ValidationError> {
    if manifest.schema_version != "2.0" && manifest.schema_version != "2.1" {
        return Err(ValidationError::UnsupportedSchema(
            manifest.schema_version.clone(),
            "2.0 или 2.1",
        ));
    }

    if manifest.id.trim().is_empty() {
        return Err(ValidationError::EmptyField("id"));
    }

    if manifest.name.trim().is_empty() {
        return Err(ValidationError::EmptyField("name"));
    }

    if manifest.version.trim().is_empty() {
        return Err(ValidationError::EmptyField("version"));
    }

    // Валидация стратегий
    for (name, strat) in &manifest.strategies {
        if strat.preference.is_empty() {
            return Err(ValidationError::EmptyStrategy(name.clone()));
        }
    }

    // Валидация совпадений
    for rule in &manifest.matches {
        if let Some(ref strat_name) = rule.strategy {
            if !manifest.strategies.contains_key(strat_name) {
                return Err(ValidationError::UndefinedStrategy(strat_name.clone()));
            }
        }

        for domain in &rule.domains {
            validate_domain_syntax(domain)?;

            if !manifest.allow_sensitive {
                let clean = clean_domain_for_check(domain);
                for protected in PROTECTED_SYSTEM_DOMAINS {
                    if clean == *protected || clean.ends_with(&format!(".{}", protected)) {
                        return Err(ValidationError::SensitiveDomainBlocked(domain.clone()));
                    }
                }
            }
        }
    }

    Ok(())
}

fn clean_domain_for_check(domain: &str) -> String {
    domain
        .trim()
        .trim_start_matches('*')
        .trim_start_matches('.')
        .to_ascii_lowercase()
}

fn validate_domain_syntax(domain: &str) -> Result<(), ValidationError> {
    let d = domain.trim();
    if d.is_empty() {
        return Err(ValidationError::InvalidDomain(
            domain.to_string(),
            "пустая строка",
        ));
    }

    if d.contains('/') || d.contains(':') || d.contains(' ') || d.contains('\\') {
        return Err(ValidationError::InvalidDomain(
            domain.to_string(),
            "содержит запрещенные символы (слэш, двоеточие, пробел)",
        ));
    }

    let core = d.trim_start_matches('*').trim_start_matches('.');
    if core.is_empty() {
        return Err(ValidationError::InvalidDomain(
            domain.to_string(),
            "не содержит имени хоста",
        ));
    }

    Ok(())
}

/// Анализатор взаимного перекрытия (shadowing) правил в цепочке
pub fn detect_shadowing(manifest: &RuleManifest) -> Vec<ShadowingWarning> {
    let mut warnings = Vec::new();

    for (i, rule_a) in manifest.matches.iter().enumerate() {
        for (j, rule_b) in manifest.matches.iter().enumerate().skip(i + 1) {
            for dom_a in &rule_a.domains {
                for dom_b in &rule_b.domains {
                    let clean_a = clean_domain_for_check(dom_a);
                    let clean_b = clean_domain_for_check(dom_b);

                    let a_is_wildcard = dom_a.starts_with('*');

                    // Если правило A - это wildcard (*.domain.com), а B - конкретный поддомен (sub.domain.com)
                    if a_is_wildcard && (clean_b == clean_a || clean_b.ends_with(&format!(".{}", clean_a))) {
                        warnings.push(ShadowingWarning {
                            broader_rule_index: i,
                            shadowed_rule_index: j,
                            broader_pattern: dom_a.clone(),
                            shadowed_pattern: dom_b.clone(),
                            message: format!(
                                "Правило #{} ('{}') перекрывает более специфичное правило #{} ('{}'), расположенное ниже",
                                i + 1, dom_a, j + 1, dom_b
                            ),
                        });
                    } else if !a_is_wildcard && dom_a == dom_b {
                        warnings.push(ShadowingWarning {
                            broader_rule_index: i,
                            shadowed_rule_index: j,
                            broader_pattern: dom_a.clone(),
                            shadowed_pattern: dom_b.clone(),
                            message: format!(
                                "Правило #{} содержит дубликат домена ('{}'), уже обработанного в правиле #{}",
                                j + 1, dom_b, i + 1
                            ),
                        });
                    }
                }
            }
        }
    }

    warnings
}

