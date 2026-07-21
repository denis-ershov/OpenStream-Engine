//! Минимальное XML-дерево для MPD (owned, round-trip friendly).

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::io::Cursor;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DashError {
    #[error("xml: {0}")]
    Xml(String),
    #[error("expected root MPD element")]
    NotMpd,
}

#[derive(Debug, Clone)]
pub struct XmlElement {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<XmlNode>,
}

#[derive(Debug, Clone)]
pub enum XmlNode {
    Element(XmlElement),
    Text(String),
}

impl XmlElement {
    pub fn attr(&self, key: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }

    pub fn local_name(&self) -> &str {
        self.name
            .rsplit(':')
            .next()
            .unwrap_or(self.name.as_str())
    }

    pub fn children_elems_mut(&mut self) -> impl Iterator<Item = &mut XmlElement> {
        self.children.iter_mut().filter_map(|n| match n {
            XmlNode::Element(e) => Some(e),
            XmlNode::Text(_) => None,
        })
    }

    pub fn children_elems(&self) -> impl Iterator<Item = &XmlElement> {
        self.children.iter().filter_map(|n| match n {
            XmlNode::Element(e) => Some(e),
            XmlNode::Text(_) => None,
        })
    }

    pub fn find_children(&self, local: &str) -> Vec<&XmlElement> {
        self.children_elems()
            .filter(|e| e.local_name().eq_ignore_ascii_case(local))
            .collect()
    }

    pub fn text_content(&self) -> String {
        let mut out = String::new();
        for c in &self.children {
            match c {
                XmlNode::Text(t) => out.push_str(t),
                XmlNode::Element(e) => out.push_str(&e.text_content()),
            }
        }
        out
    }
}

fn local_name(qname: &[u8]) -> String {
    let s = String::from_utf8_lossy(qname);
    s.rsplit(':').next().unwrap_or(&s).to_string()
}

fn attrs_from_start(e: &BytesStart<'_>) -> Result<Vec<(String, String)>, DashError> {
    let mut out = Vec::new();
    for a in e.attributes().with_checks(false) {
        let a = a.map_err(|err| DashError::Xml(err.to_string()))?;
        let key = String::from_utf8_lossy(a.key.as_ref()).into_owned();
        let val = a
            .unescape_value()
            .map_err(|err| DashError::Xml(err.to_string()))?
            .into_owned();
        out.push((key, val));
    }
    Ok(out)
}

/// Парсит XML в дерево. Корневой элемент возвращается как есть (ожидается MPD).
pub fn parse_xml(input: &str) -> Result<XmlElement, DashError> {
    let mut reader = Reader::from_str(input);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut stack: Vec<XmlElement> = Vec::new();
    let mut root: Option<XmlElement> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = local_name(e.name().as_ref());
                let attrs = attrs_from_start(&e)?;
                stack.push(XmlElement {
                    name,
                    attrs,
                    children: Vec::new(),
                });
            }
            Ok(Event::Empty(e)) => {
                let name = local_name(e.name().as_ref());
                let attrs = attrs_from_start(&e)?;
                let el = XmlElement {
                    name,
                    attrs,
                    children: Vec::new(),
                };
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(XmlNode::Element(el));
                } else if root.is_none() {
                    root = Some(el);
                }
            }
            Ok(Event::End(_)) => {
                let el = stack
                    .pop()
                    .ok_or_else(|| DashError::Xml("unexpected end".into()))?;
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(XmlNode::Element(el));
                } else {
                    root = Some(el);
                }
            }
            Ok(Event::Text(t)) => {
                let text = t
                    .unescape()
                    .map_err(|err| DashError::Xml(err.to_string()))?
                    .into_owned();
                if text.is_empty() {
                    buf.clear();
                    continue;
                }
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(XmlNode::Text(text));
                }
            }
            Ok(Event::CData(t)) => {
                let text = String::from_utf8_lossy(&t).into_owned();
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(XmlNode::Text(text));
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => return Err(DashError::Xml(e.to_string())),
        }
        buf.clear();
    }

    root.ok_or(DashError::NotMpd)
}

pub fn serialize_xml(root: &XmlElement) -> Result<String, DashError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    write_element(&mut writer, root)?;
    let bytes = writer.into_inner().into_inner();
    String::from_utf8(bytes).map_err(|e| DashError::Xml(e.to_string()))
}

fn write_element(writer: &mut Writer<Cursor<Vec<u8>>>, el: &XmlElement) -> Result<(), DashError> {
    let mut start = BytesStart::new(el.name.as_str());
    for (k, v) in &el.attrs {
        start.push_attribute((k.as_str(), v.as_str()));
    }
    if el.children.is_empty() {
        writer
            .write_event(Event::Empty(start))
            .map_err(|e| DashError::Xml(e.to_string()))?;
        return Ok(());
    }
    writer
        .write_event(Event::Start(start))
        .map_err(|e| DashError::Xml(e.to_string()))?;
    for child in &el.children {
        match child {
            XmlNode::Element(e) => write_element(writer, e)?,
            XmlNode::Text(t) => {
                writer
                    .write_event(Event::Text(BytesText::new(t)))
                    .map_err(|e| DashError::Xml(e.to_string()))?;
            }
        }
    }
    writer
        .write_event(Event::End(BytesEnd::new(el.name.as_str())))
        .map_err(|e| DashError::Xml(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_simple() {
        let xml = r#"<MPD type="static"><Period id="p0"><AdaptationSet contentType="video"/></Period></MPD>"#;
        let root = parse_xml(xml).unwrap();
        assert_eq!(root.local_name(), "MPD");
        assert_eq!(root.attr("type"), Some("static"));
        let out = serialize_xml(&root).unwrap();
        assert!(out.contains("Period"));
        assert!(out.contains("AdaptationSet"));
    }
}
