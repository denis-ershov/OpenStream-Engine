//! HLS Manifest Engine: парсинг и сериализация m3u8.

use std::fmt;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("empty manifest")]
    Empty,
    #[error("missing #EXTM3U")]
    MissingHeader,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlaylistKind {
    Master,
    Media,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtInf {
    pub duration: f64,
    /// Заголовок после запятой, например `live` или пусто.
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    ExtM3U,
    Version(u32),
    TargetDuration(u32),
    MediaSequence(u64),
    Discontinuity,
    ProgramDateTime(String),
    DateRange(String),
    Prefetch(String),
    TwitchPrefetch(String),
    EndList,
    StreamInf(String),
    ExtInf(ExtInf),
    /// Неизвестный/прочий тег целиком (включая `#`).
    Opaque(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    Tag(Tag),
    /// URI сегмента или варианта (следующая строка после URI-тега).
    Uri(String),
    Comment(String),
    Blank,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Manifest {
    pub entries: Vec<Entry>,
}

impl Manifest {
    pub fn kind(&self) -> PlaylistKind {
        let has_stream_inf = self.entries.iter().any(|e| {
            matches!(e, Entry::Tag(Tag::StreamInf(_)))
        });
        let has_extinf = self.entries.iter().any(|e| {
            matches!(e, Entry::Tag(Tag::ExtInf(_)))
        });
        if has_stream_inf {
            PlaylistKind::Master
        } else if has_extinf {
            PlaylistKind::Media
        } else {
            PlaylistKind::Unknown
        }
    }

    pub fn media_sequence(&self) -> Option<u64> {
        self.entries.iter().find_map(|e| match e {
            Entry::Tag(Tag::MediaSequence(v)) => Some(*v),
            _ => None,
        })
    }

    pub fn set_media_sequence(&mut self, seq: u64) {
        for e in &mut self.entries {
            if let Entry::Tag(Tag::MediaSequence(v)) = e {
                *v = seq;
                return;
            }
        }
        // Вставить после EXTM3U / VERSION если тега не было.
        let insert_at = self
            .entries
            .iter()
            .position(|e| !matches!(e, Entry::Tag(Tag::ExtM3U) | Entry::Tag(Tag::Version(_))))
            .unwrap_or(self.entries.len());
        self.entries
            .insert(insert_at, Entry::Tag(Tag::MediaSequence(seq)));
    }

    /// Индексы пар (ExtInf index, Uri index) для media-сегментов.
    pub fn segment_pairs(&self) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        let mut i = 0;
        while i < self.entries.len() {
            if matches!(&self.entries[i], Entry::Tag(Tag::ExtInf(_))) {
                let mut j = i + 1;
                while j < self.entries.len() {
                    match &self.entries[j] {
                        Entry::Uri(_) => {
                            pairs.push((i, j));
                            break;
                        }
                        Entry::Blank | Entry::Comment(_) => j += 1,
                        Entry::Tag(Tag::ProgramDateTime(_))
                        | Entry::Tag(Tag::Discontinuity)
                        | Entry::Tag(Tag::Opaque(_)) => j += 1,
                        _ => break,
                    }
                }
            }
            i += 1;
        }
        pairs
    }
}

pub fn parse(input: &str) -> Result<Manifest, ManifestError> {
    if input.trim().is_empty() {
        return Err(ManifestError::Empty);
    }
    let mut entries = Vec::new();
    let mut saw_header = false;

    for line in input.lines() {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            entries.push(Entry::Blank);
            continue;
        }
        if line.starts_with('#') {
            if line == "#EXTM3U" {
                saw_header = true;
                entries.push(Entry::Tag(Tag::ExtM3U));
            } else if let Some(rest) = line.strip_prefix("#EXT-X-VERSION:") {
                entries.push(Entry::Tag(Tag::Version(rest.trim().parse().unwrap_or(0))));
            } else if let Some(rest) = line.strip_prefix("#EXT-X-TARGETDURATION:") {
                entries.push(Entry::Tag(Tag::TargetDuration(
                    rest.trim().parse().unwrap_or(0),
                )));
            } else if let Some(rest) = line.strip_prefix("#EXT-X-MEDIA-SEQUENCE:") {
                entries.push(Entry::Tag(Tag::MediaSequence(
                    rest.trim().parse().unwrap_or(0),
                )));
            } else if line == "#EXT-X-DISCONTINUITY" {
                entries.push(Entry::Tag(Tag::Discontinuity));
            } else if let Some(rest) = line.strip_prefix("#EXT-X-PROGRAM-DATE-TIME:") {
                entries.push(Entry::Tag(Tag::ProgramDateTime(rest.to_string())));
            } else if let Some(rest) = line.strip_prefix("#EXT-X-DATERANGE:") {
                entries.push(Entry::Tag(Tag::DateRange(rest.to_string())));
            } else if let Some(rest) = line.strip_prefix("#EXT-X-TWITCH-PREFETCH:") {
                entries.push(Entry::Tag(Tag::TwitchPrefetch(rest.to_string())));
            } else if let Some(rest) = line.strip_prefix("#EXT-X-PREFETCH:") {
                entries.push(Entry::Tag(Tag::Prefetch(rest.to_string())));
            } else if line == "#EXT-X-ENDLIST" {
                entries.push(Entry::Tag(Tag::EndList));
            } else if let Some(rest) = line.strip_prefix("#EXT-X-STREAM-INF:") {
                entries.push(Entry::Tag(Tag::StreamInf(rest.to_string())));
            } else if let Some(rest) = line.strip_prefix("#EXTINF:") {
                entries.push(Entry::Tag(Tag::ExtInf(parse_extinf(rest))));
            } else if line.starts_with("#EXT") {
                entries.push(Entry::Tag(Tag::Opaque(line.to_string())));
            } else {
                entries.push(Entry::Comment(line.to_string()));
            }
        } else {
            entries.push(Entry::Uri(line.to_string()));
        }
    }

    if !saw_header {
        return Err(ManifestError::MissingHeader);
    }
    Ok(Manifest { entries })
}

fn parse_extinf(rest: &str) -> ExtInf {
    let (dur_s, title) = match rest.split_once(',') {
        Some((d, t)) => (d.trim(), {
            let t = t.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        }),
        None => (rest.trim(), None),
    };
    let duration = dur_s.trim_end_matches(',').parse().unwrap_or(0.0);
    ExtInf { duration, title }
}

pub fn serialize(manifest: &Manifest) -> String {
    let mut out = String::with_capacity(manifest.entries.len() * 32);
    for (idx, entry) in manifest.entries.iter().enumerate() {
        match entry {
            Entry::Blank => {}
            Entry::Comment(c) => out.push_str(c),
            Entry::Uri(u) => out.push_str(u),
            Entry::Tag(t) => out.push_str(&tag_to_string(t)),
        }
        if idx + 1 < manifest.entries.len() {
            out.push('\n');
        }
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn tag_to_string(tag: &Tag) -> String {
    match tag {
        Tag::ExtM3U => "#EXTM3U".into(),
        Tag::Version(v) => format!("#EXT-X-VERSION:{v}"),
        Tag::TargetDuration(v) => format!("#EXT-X-TARGETDURATION:{v}"),
        Tag::MediaSequence(v) => format!("#EXT-X-MEDIA-SEQUENCE:{v}"),
        Tag::Discontinuity => "#EXT-X-DISCONTINUITY".into(),
        Tag::ProgramDateTime(s) => format!("#EXT-X-PROGRAM-DATE-TIME:{s}"),
        Tag::DateRange(s) => format!("#EXT-X-DATERANGE:{s}"),
        Tag::Prefetch(s) => format!("#EXT-X-PREFETCH:{s}"),
        Tag::TwitchPrefetch(s) => format!("#EXT-X-TWITCH-PREFETCH:{s}"),
        Tag::EndList => "#EXT-X-ENDLIST".into(),
        Tag::StreamInf(s) => format!("#EXT-X-STREAM-INF:{s}"),
        Tag::ExtInf(e) => match &e.title {
            Some(t) => format!("#EXTINF:{:.3},{}", e.duration, t),
            None => format!("#EXTINF:{:.3},", e.duration),
        },
        Tag::Opaque(s) => s.clone(),
    }
}

impl fmt::Display for Manifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&serialize(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_media() {
        let raw = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:2
#EXT-X-MEDIA-SEQUENCE:100
#EXTINF:2.000,live
seg100.ts
#EXTINF:2.000,live
seg101.ts
"#;
        let m = parse(raw).unwrap();
        assert_eq!(m.kind(), PlaylistKind::Media);
        assert_eq!(m.media_sequence(), Some(100));
        assert_eq!(m.segment_pairs().len(), 2);
        let again = parse(&serialize(&m)).unwrap();
        assert_eq!(again.segment_pairs().len(), 2);
    }

    #[test]
    fn parse_master() {
        let raw = r#"#EXTM3U
#EXT-X-STREAM-INF:BANDWIDTH=1000000
low.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=3000000
high.m3u8
"#;
        let m = parse(raw).unwrap();
        assert_eq!(m.kind(), PlaylistKind::Master);
    }

    #[test]
    fn parse_prefetch_and_daterange() {
        let raw = r#"#EXTM3U
#EXT-X-MEDIA-SEQUENCE:1
#EXT-X-DATERANGE:ID="stitched-ad-1",CLASS="twitch-stitched-ad"
#EXTINF:2.000,
ad1.ts
#EXT-X-TWITCH-PREFETCH:prefetch.ts
#EXTINF:2.000,live
live1.ts
"#;
        let m = parse(raw).unwrap();
        assert!(m.entries.iter().any(|e| matches!(e, Entry::Tag(Tag::DateRange(_)))));
        assert!(m
            .entries
            .iter()
            .any(|e| matches!(e, Entry::Tag(Tag::TwitchPrefetch(_)))));
    }
}
