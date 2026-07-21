//! Segment Engine: классификация URL без изменения содержимого (HLS + CMAF/DASH).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentKind {
    TransportStream,
    /// CMAF / fMP4 media segment (`.m4s`, `.cmfv`, `.cmfa`).
    FragmentedMp4,
    /// Init segment часто `.mp4` / `.m4s` без media; тело не трогаем.
    Mp4,
    Aac,
    Webm,
    /// HLS playlist.
    PlaylistHls,
    /// DASH MPD.
    PlaylistDash,
    Other,
}

pub fn classify_uri(uri: &str) -> SegmentKind {
    let path = uri.split('?').next().unwrap_or(uri).to_lowercase();
    if path.ends_with(".m3u8") {
        SegmentKind::PlaylistHls
    } else if path.ends_with(".mpd") {
        SegmentKind::PlaylistDash
    } else if path.ends_with(".ts") {
        SegmentKind::TransportStream
    } else if path.ends_with(".m4s")
        || path.ends_with(".cmfv")
        || path.ends_with(".cmfa")
        || path.ends_with(".cmft")
    {
        SegmentKind::FragmentedMp4
    } else if path.ends_with(".mp4") || path.ends_with(".m4v") || path.ends_with(".m4a") {
        SegmentKind::Mp4
    } else if path.ends_with(".webm") {
        SegmentKind::Webm
    } else if path.ends_with(".aac") {
        SegmentKind::Aac
    } else {
        SegmentKind::Other
    }
}

pub fn is_playlist(uri: &str) -> bool {
    matches!(
        classify_uri(uri),
        SegmentKind::PlaylistHls | SegmentKind::PlaylistDash
    )
}

pub fn is_media_segment(uri: &str) -> bool {
    matches!(
        classify_uri(uri),
        SegmentKind::TransportStream
            | SegmentKind::FragmentedMp4
            | SegmentKind::Mp4
            | SegmentKind::Aac
            | SegmentKind::Webm
    )
}

/// CMAF media/init: только passthrough (не буферизовать как манифест).
pub fn is_cmaf_segment(uri: &str) -> bool {
    matches!(
        classify_uri(uri),
        SegmentKind::FragmentedMp4 | SegmentKind::Mp4
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify() {
        assert_eq!(classify_uri("a.ts?token=1"), SegmentKind::TransportStream);
        assert_eq!(classify_uri("x.m3u8"), SegmentKind::PlaylistHls);
        assert_eq!(classify_uri("live.mpd"), SegmentKind::PlaylistDash);
        assert!(is_media_segment("seg.m4s"));
        assert!(is_cmaf_segment("init.mp4"));
        assert!(is_playlist("/a.mpd?x=1"));
        assert!(!is_media_segment("/a.mpd"));
    }
}
