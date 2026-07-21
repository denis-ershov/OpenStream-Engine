//! Ad Detector: Rules → Markers → Confidence.

use ose_manifest::{Entry, ExtInf, Manifest, Tag};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleKind {
    /// Строка содержит подстроку (например `stitched`).
    Contains,
    /// Тег EXT-X-DATERANGE (pattern = подстрока в атрибутах, обычно `stitched`).
    DateRange,
    /// EXTINF без title `live`.
    ExtInfNotLive,
    /// Настоящий regex по атрибутам/title.
    Regex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub kind: RuleKind,
    pub pattern: String,
    pub enabled: bool,
}

impl Rule {
    pub fn contains(pattern: impl Into<String>) -> Self {
        Self {
            kind: RuleKind::Contains,
            pattern: pattern.into(),
            enabled: true,
        }
    }

    pub fn date_range_stitched() -> Self {
        Self {
            kind: RuleKind::DateRange,
            pattern: "stitched".into(),
            enabled: true,
        }
    }

    pub fn extinf_not_live() -> Self {
        Self {
            kind: RuleKind::ExtInfNotLive,
            pattern: "live".into(),
            enabled: true,
        }
    }

    pub fn regex(pattern: impl Into<String>) -> Self {
        Self {
            kind: RuleKind::Regex,
            pattern: pattern.into(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Marker {
    pub entry_index: usize,
    pub rule: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub markers: Vec<Marker>,
    /// 0.0..=1.0
    pub confidence: f32,
    /// Индексы ExtInf, помеченные как реклама.
    pub ad_extinf_indices: Vec<usize>,
}

pub fn default_twitch_rules() -> Vec<Rule> {
    vec![
        Rule::contains("stitched"),
        Rule::date_range_stitched(),
        Rule::extinf_not_live(),
    ]
}

fn compiled_regex(pattern: &str) -> Option<&'static Regex> {
    // Кэш одного типового паттерна; иначе компилируем на месте через thread-local map упрощённо.
    static AD_RE: OnceLock<Regex> = OnceLock::new();
    if pattern == r"(?i)stitched|twitch-stitched-ad|X-TV-TWITCH-AD" {
        return Some(AD_RE.get_or_init(|| {
            Regex::new(r"(?i)stitched|twitch-stitched-ad|X-TV-TWITCH-AD").expect("valid regex")
        }));
    }
    None
}

fn regex_matches(pattern: &str, haystack: &str) -> bool {
    if let Some(re) = compiled_regex(pattern) {
        return re.is_match(haystack);
    }
    Regex::new(pattern)
        .map(|re| re.is_match(haystack))
        .unwrap_or(false)
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    haystack.to_ascii_lowercase().contains(&needle.to_ascii_lowercase())
}

/// DATERANGE считается рекламным только по явным Twitch/stitched маркерам.
pub fn is_ad_daterange(attrs: &str) -> bool {
    let lower = attrs.to_ascii_lowercase();
    lower.contains("stitched")
        || lower.contains("twitch-stitched-ad")
        || lower.contains("x-tv-twitch-ad")
}

pub fn detect(manifest: &Manifest, rules: &[Rule]) -> Detection {
    let mut markers = Vec::new();
    let mut ad_extinf = Vec::new();
    let enabled: Vec<&Rule> = rules.iter().filter(|r| r.enabled).collect();

    let mut in_ad_block = false;

    for (idx, entry) in manifest.entries.iter().enumerate() {
        match entry {
            Entry::Tag(Tag::DateRange(attrs)) => {
                for rule in &enabled {
                    let hit = match rule.kind {
                        RuleKind::DateRange => {
                            if rule.pattern.is_empty() {
                                is_ad_daterange(attrs)
                            } else {
                                contains_ci(attrs, &rule.pattern)
                            }
                        }
                        RuleKind::Contains => contains_ci(attrs, &rule.pattern),
                        RuleKind::Regex => regex_matches(&rule.pattern, attrs),
                        RuleKind::ExtInfNotLive => false,
                    };
                    if hit {
                        markers.push(Marker {
                            entry_index: idx,
                            rule: format!("{:?}", rule.kind),
                            detail: attrs.clone(),
                        });
                        if is_ad_daterange(attrs) || contains_ci(attrs, "stitched") {
                            in_ad_block = true;
                        }
                    }
                }
            }
            Entry::Tag(Tag::Opaque(s)) | Entry::Comment(s) => {
                for rule in &enabled {
                    let hit = match rule.kind {
                        RuleKind::Contains => contains_ci(s, &rule.pattern),
                        RuleKind::Regex => regex_matches(&rule.pattern, s),
                        _ => false,
                    };
                    if hit {
                        markers.push(Marker {
                            entry_index: idx,
                            rule: format!("{:?}", rule.kind),
                            detail: s.clone(),
                        });
                        if contains_ci(s, "stitched") {
                            in_ad_block = true;
                        }
                    }
                }
            }
            Entry::Tag(Tag::ExtInf(info)) => {
                let not_live = !is_live(info);
                let mut is_ad = false;
                let next_uri = manifest.entries.get(idx + 1).and_then(|e| match e {
                    Entry::Uri(u) => Some(u.as_str()),
                    _ => None,
                });

                for rule in &enabled {
                    match rule.kind {
                        RuleKind::ExtInfNotLive if not_live => {
                            is_ad = true;
                            markers.push(Marker {
                                entry_index: idx,
                                rule: "ExtInfNotLive".into(),
                                detail: info.title.clone().unwrap_or_default(),
                            });
                        }
                        RuleKind::Contains => {
                            let title_hit = info
                                .title
                                .as_ref()
                                .is_some_and(|t| contains_ci(t, &rule.pattern));
                            let uri_hit = next_uri.is_some_and(|u| contains_ci(u, &rule.pattern));
                            if title_hit || uri_hit {
                                is_ad = true;
                                markers.push(Marker {
                                    entry_index: idx,
                                    rule: "Contains".into(),
                                    detail: info
                                        .title
                                        .clone()
                                        .or_else(|| next_uri.map(str::to_string))
                                        .unwrap_or_default(),
                                });
                            }
                        }
                        RuleKind::Regex => {
                            let title_hit = info
                                .title
                                .as_ref()
                                .is_some_and(|t| regex_matches(&rule.pattern, t));
                            let uri_hit =
                                next_uri.is_some_and(|u| regex_matches(&rule.pattern, u));
                            if title_hit || uri_hit {
                                is_ad = true;
                                markers.push(Marker {
                                    entry_index: idx,
                                    rule: "Regex".into(),
                                    detail: info
                                        .title
                                        .clone()
                                        .or_else(|| next_uri.map(str::to_string))
                                        .unwrap_or_default(),
                                });
                            }
                        }
                        _ => {}
                    }
                }

                if in_ad_block && not_live {
                    is_ad = true;
                }
                if is_live(info) {
                    in_ad_block = false;
                }
                if is_ad {
                    ad_extinf.push(idx);
                }
            }
            _ => {}
        }
    }

    ad_extinf.sort_unstable();
    ad_extinf.dedup();

    let confidence = if ad_extinf.is_empty() {
        0.0
    } else if markers
        .iter()
        .any(|m| contains_ci(&m.detail, "stitched") || contains_ci(&m.detail, "twitch-stitched"))
    {
        0.95
    } else if markers.iter().any(|m| m.rule == "ExtInfNotLive") {
        0.8
    } else {
        0.6
    };

    Detection {
        markers,
        confidence,
        ad_extinf_indices: ad_extinf,
    }
}

fn is_live(info: &ExtInf) -> bool {
    info.title
        .as_deref()
        .map(|t| t.split(',').any(|p| p.trim() == "live") || t.trim() == "live")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ose_manifest::parse;

    #[test]
    fn detect_stitched_ad() {
        let raw = r#"#EXTM3U
#EXT-X-MEDIA-SEQUENCE:101
#EXTINF:2.000,live
seg101.ts
#EXT-X-DATERANGE:ID="stitched-ad-1",CLASS="twitch-stitched-ad"
#EXTINF:2.000,
ad201.ts
#EXTINF:2.000,
ad202.ts
#EXTINF:2.000,live
seg103.ts
"#;
        let m = parse(raw).unwrap();
        let d = detect(&m, &default_twitch_rules());
        assert!(d.confidence >= 0.8);
        assert_eq!(d.ad_extinf_indices.len(), 2);
    }

    #[test]
    fn regex_rule_matches() {
        let raw = r#"#EXTM3U
#EXT-X-MEDIA-SEQUENCE:1
#EXT-X-DATERANGE:CLASS="twitch-stitched-ad",X-TV-TWITCH-AD-ROLL-TYPE="MIDROLL"
#EXTINF:2.000,
ad.ts
#EXTINF:2.000,live
live.ts
"#;
        let m = parse(raw).unwrap();
        let rules = vec![Rule::regex(r"(?i)X-TV-TWITCH-AD")];
        let d = detect(&m, &rules);
        assert!(!d.markers.is_empty());
    }

    #[test]
    fn broad_ad_word_not_enough_for_daterange_helper() {
        assert!(!is_ad_daterange(r#"ID="foo",CLASS="metadata""#));
        assert!(is_ad_daterange(r#"CLASS="twitch-stitched-ad""#));
    }
}
