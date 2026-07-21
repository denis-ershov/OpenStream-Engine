//! Типизированный взгляд на MPD поверх XmlElement.

use crate::xml::{serialize_xml, DashError, XmlElement};

#[derive(Debug)]
pub struct Mpd {
    pub root: XmlElement,
}

impl Mpd {
    pub fn parse(input: &str) -> Result<Self, DashError> {
        let root = crate::xml::parse_xml(input)?;
        if !root.local_name().eq_ignore_ascii_case("MPD") {
            return Err(DashError::NotMpd);
        }
        Ok(Self { root })
    }

    pub fn serialize(&self) -> Result<String, DashError> {
        serialize_xml(&self.root)
    }

    pub fn periods(&self) -> Vec<PeriodView<'_>> {
        self.root
            .find_children("Period")
            .into_iter()
            .map(PeriodView)
            .collect()
    }

    pub fn period_count(&self) -> usize {
        self.root
            .children_elems()
            .filter(|e| e.local_name().eq_ignore_ascii_case("Period"))
            .count()
    }

    /// Удаляет Period-элементы по индексам среди Period-детей (не среди всех children).
    pub fn remove_periods_by_indices(&mut self, indices: &[usize]) {
        let mut period_i = 0usize;
        self.root.children.retain(|n| match n {
            crate::xml::XmlNode::Element(e) if e.local_name().eq_ignore_ascii_case("Period") => {
                let keep = !indices.contains(&period_i);
                period_i += 1;
                keep
            }
            _ => true,
        });
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PeriodView<'a>(pub &'a XmlElement);

impl PeriodView<'_> {
    pub fn id(&self) -> Option<&str> {
        self.0.attr("id")
    }

    pub fn adaptation_sets(&self) -> Vec<AdaptationView<'_>> {
        self.0
            .find_children("AdaptationSet")
            .into_iter()
            .map(AdaptationView)
            .collect()
    }

    pub fn descriptors(&self, local: &str) -> Vec<&XmlElement> {
        self.0.find_children(local)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AdaptationView<'a>(pub &'a XmlElement);

impl AdaptationView<'_> {
    pub fn id(&self) -> Option<&str> {
        self.0.attr("id")
    }

    pub fn content_type(&self) -> Option<&str> {
        self.0.attr("contentType")
    }

    pub fn mime_type(&self) -> Option<&str> {
        self.0.attr("mimeType")
    }
}
