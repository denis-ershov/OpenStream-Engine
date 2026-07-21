//! Общий контракт фильтрации медиа-манифестов (HLS Entry / DASH Node).

use ose_dash::Mpd;
use ose_manifest::Manifest;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MediaError {
    #[error("{0}")]
    Msg(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestKind {
    Hls,
    Dash,
    Unknown,
}

impl ManifestKind {
    pub fn from_path(path: &str) -> Self {
        let p = path
            .split('?')
            .next()
            .unwrap_or(path)
            .to_ascii_lowercase();
        if p.ends_with(".m3u8") {
            Self::Hls
        } else if p.ends_with(".mpd") {
            Self::Dash
        } else {
            Self::Unknown
        }
    }

    pub fn is_manifest(self) -> bool {
        matches!(self, Self::Hls | Self::Dash)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterOutcome {
    pub ads_found: bool,
    /// HLS: сегменты; DASH: Period + AdaptationSet.
    pub units_removed: u64,
}

/// Унифицированный фильтр: HLS AST и DASH MPD.
pub trait MediaFilter: Send + Sync {
    fn name(&self) -> &str;

    fn apply_hls(&self, _manifest: &mut Manifest) -> Result<FilterOutcome, MediaError> {
        Ok(FilterOutcome::default())
    }

    fn apply_dash(&self, _mpd: &mut Mpd) -> Result<FilterOutcome, MediaError> {
        Ok(FilterOutcome::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_from_path() {
        assert_eq!(ManifestKind::from_path("/a/b.m3u8?x=1"), ManifestKind::Hls);
        assert_eq!(ManifestKind::from_path("/live.mpd"), ManifestKind::Dash);
        assert_eq!(ManifestKind::from_path("/seg.m4s"), ManifestKind::Unknown);
    }
}
