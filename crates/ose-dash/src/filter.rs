//! Эвристики рекламных Period / AdaptationSet.

use serde::{Deserialize, Serialize};

use crate::mpd::Mpd;
use crate::xml::XmlElement;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashFilterRules {
    /// Подстроки в id/label Period (case-insensitive).
    #[serde(default = "default_period_markers")]
    pub period_id_markers: Vec<String>,
    /// schemeIdUri / value AssetIdentifier / EssentialProperty / SupplementalProperty.
    #[serde(default = "default_scheme_markers")]
    pub scheme_markers: Vec<String>,
    /// Удалять AdaptationSet с Role=… содержащим маркеры.
    #[serde(default = "default_role_markers")]
    pub role_markers: Vec<String>,
    #[serde(default = "default_true")]
    pub remove_ad_periods: bool,
    #[serde(default = "default_true")]
    pub remove_ad_adaptations: bool,
}

fn default_true() -> bool {
    true
}

fn default_period_markers() -> Vec<String> {
    vec![
        "ad".into(),
        "ads".into(),
        "advert".into(),
        "midroll".into(),
        "preroll".into(),
        "break".into(),
        "commercial".into(),
    ]
}

fn default_scheme_markers() -> Vec<String> {
    vec![
        "advertising".into(),
        "urn:scte:dash:ad".into(),
        "urn:mpeg:dash:period:ad".into(),
        "schemeIdUri=\"urn:scte".into(),
        "ad-id".into(),
        "dai".into(),
    ]
}

fn default_role_markers() -> Vec<String> {
    vec!["advertisement".into(), "ad".into()]
}

impl Default for DashFilterRules {
    fn default() -> Self {
        Self {
            period_id_markers: default_period_markers(),
            scheme_markers: default_scheme_markers(),
            role_markers: default_role_markers(),
            remove_ad_periods: true,
            remove_ad_adaptations: true,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterStats {
    pub periods_removed: u64,
    pub adaptations_removed: u64,
}

impl FilterStats {
    pub fn any_removed(&self) -> bool {
        self.periods_removed > 0 || self.adaptations_removed > 0
    }
}

fn contains_any(hay: &str, needles: &[String]) -> bool {
    let h = hay.to_ascii_lowercase();
    needles.iter().any(|n| {
        let n = n.to_ascii_lowercase();
        !n.is_empty() && h.contains(&n)
    })
}

fn element_blob(el: &XmlElement) -> String {
    let mut s = String::new();
    if let Some(id) = el.attr("id") {
        s.push_str(id);
        s.push(' ');
    }
    for (k, v) in &el.attrs {
        s.push_str(k);
        s.push('=');
        s.push_str(v);
        s.push(' ');
    }
    s.push_str(&el.text_content());
    for child in el.children_elems() {
        s.push_str(&element_blob(child));
        s.push(' ');
    }
    s
}

pub fn is_ad_period(period: &XmlElement, rules: &DashFilterRules) -> bool {
    if let Some(id) = period.attr("id") {
        if contains_any(id, &rules.period_id_markers) {
            return true;
        }
    }
    let blob = element_blob(period);
    contains_any(&blob, &rules.scheme_markers)
        || period
            .find_children("AssetIdentifier")
            .iter()
            .any(|d| contains_any(&element_blob(d), &rules.scheme_markers))
        || period
            .find_children("SupplementalProperty")
            .iter()
            .any(|d| contains_any(&element_blob(d), &rules.scheme_markers))
        || period
            .find_children("EssentialProperty")
            .iter()
            .any(|d| contains_any(&element_blob(d), &rules.scheme_markers))
}

pub fn is_ad_adaptation(aset: &XmlElement, rules: &DashFilterRules) -> bool {
    for role in aset.find_children("Role") {
        let blob = element_blob(role);
        if contains_any(&blob, &rules.role_markers) {
            return true;
        }
    }
    let blob = element_blob(aset);
    contains_any(&blob, &rules.scheme_markers)
}

/// Удаляет рекламные Period и/или AdaptationSet по правилам.
pub fn filter_ad_nodes(mpd: &mut Mpd, rules: &DashFilterRules) -> FilterStats {
    let mut stats = FilterStats::default();

    if rules.remove_ad_periods {
        let mut remove_idx = Vec::new();
        for (i, period) in mpd
            .root
            .children_elems()
            .filter(|e| e.local_name().eq_ignore_ascii_case("Period"))
            .enumerate()
        {
            if is_ad_period(period, rules) {
                remove_idx.push(i);
            }
        }
        stats.periods_removed = remove_idx.len() as u64;
        if !remove_idx.is_empty() {
            mpd.remove_periods_by_indices(&remove_idx);
        }
    }

    if rules.remove_ad_adaptations {
        // После удаления Period работаем с оставшимися.
        let period_indices: Vec<usize> = mpd
            .root
            .children
            .iter()
            .enumerate()
            .filter_map(|(i, n)| match n {
                crate::xml::XmlNode::Element(e)
                    if e.local_name().eq_ignore_ascii_case("Period") =>
                {
                    Some(i)
                }
                _ => None,
            })
            .collect();

        for pi in period_indices {
            let crate::xml::XmlNode::Element(period) = &mut mpd.root.children[pi] else {
                continue;
            };
            let before = period
                .children
                .iter()
                .filter(|n| matches!(n, crate::xml::XmlNode::Element(e) if e.local_name().eq_ignore_ascii_case("AdaptationSet")))
                .count();
            period.children.retain(|n| match n {
                crate::xml::XmlNode::Element(e)
                    if e.local_name().eq_ignore_ascii_case("AdaptationSet") =>
                {
                    !is_ad_adaptation(e, rules)
                }
                _ => true,
            });
            let after = period
                .children
                .iter()
                .filter(|n| matches!(n, crate::xml::XmlNode::Element(e) if e.local_name().eq_ignore_ascii_case("AdaptationSet")))
                .count();
            stats.adaptations_removed += (before - after) as u64;
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_ad_period() {
        let xml = r#"<?xml version="1.0"?>
<MPD xmlns="urn:mpeg:dash:schema:mpd:2011" type="static">
  <Period id="content_0">
    <AdaptationSet contentType="video" mimeType="video/mp4">
      <Representation id="v1" bandwidth="1000000"/>
    </AdaptationSet>
  </Period>
  <Period id="midroll_ad_1">
    <AssetIdentifier schemeIdUri="urn:scte:dash:ad" value="break"/>
    <AdaptationSet contentType="video">
      <Representation id="ad" bandwidth="500000"/>
    </AdaptationSet>
  </Period>
  <Period id="content_1">
    <AdaptationSet contentType="video">
      <Representation id="v2" bandwidth="1000000"/>
    </AdaptationSet>
  </Period>
</MPD>"#;
        let mut mpd = Mpd::parse(xml).unwrap();
        assert_eq!(mpd.period_count(), 3);
        let stats = filter_ad_nodes(&mut mpd, &DashFilterRules::default());
        assert_eq!(stats.periods_removed, 1);
        assert_eq!(mpd.period_count(), 2);
        let out = mpd.serialize().unwrap();
        assert!(!out.contains("midroll_ad_1"));
        assert!(out.contains("content_0"));
        assert!(out.contains("content_1"));
    }

    #[test]
    fn strips_ad_adaptation_role() {
        let xml = r#"<MPD type="static">
  <Period id="p0">
    <AdaptationSet contentType="video">
      <Representation id="v" bandwidth="1"/>
    </AdaptationSet>
    <AdaptationSet contentType="video" id="ad-track">
      <Role schemeIdUri="urn:mpeg:dash:role:2011" value="advertisement"/>
      <Representation id="a" bandwidth="1"/>
    </AdaptationSet>
  </Period>
</MPD>"#;
        let mut mpd = Mpd::parse(xml).unwrap();
        let stats = filter_ad_nodes(&mut mpd, &DashFilterRules::default());
        assert_eq!(stats.adaptations_removed, 1);
        let out = mpd.serialize().unwrap();
        assert!(!out.contains("advertisement"));
        assert!(out.contains("contentType=\"video\""));
    }

    #[test]
    fn fixture_scte_ad_period() {
        let xml = include_str!("../fixtures/scte_ad_period.mpd");
        let mut mpd = Mpd::parse(xml).unwrap();
        assert_eq!(mpd.period_count(), 3);
        let stats = filter_ad_nodes(&mut mpd, &DashFilterRules::default());
        assert!(stats.periods_removed >= 1);
        let out = mpd.serialize().unwrap();
        assert!(!out.contains("ad_preroll"));
        assert!(out.contains("p0"));
        assert!(out.contains("p1"));
    }
}
