//! Plugin Twitch: Segment Stripping.

use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use ose_detector::{default_twitch_rules, Rule};
use ose_manifest::{ExtInf, Manifest, PlaylistKind, Tag, Entry};
use ose_plugin::{
    rewrite_master_variant_urls, strip_ad_segments, ManifestKind, Plugin, PluginCapabilities,
    PluginError, PluginStats, ProcessOutcome, RequestMeta,
};
use tracing::debug;

pub struct TwitchPlugin {
    rules: Vec<Rule>,
    playlists: AtomicU64,
    ads_found: AtomicU64,
    segments_removed: AtomicU64,
    strip_prefetch_on_ads: bool,
    pub debug: bool,
    pub max_wait_secs: u64,
    /// Opt-in scaffold (GraphQL/token backup) — не активирует seamless по умолчанию.
    pub backup_seamless: bool,
}

impl Default for TwitchPlugin {
    fn default() -> Self {
        Self::new(default_twitch_rules())
    }
}

impl TwitchPlugin {
    pub fn new(rules: Vec<Rule>) -> Self {
        Self {
            rules,
            playlists: AtomicU64::new(0),
            ads_found: AtomicU64::new(0),
            segments_removed: AtomicU64::new(0),
            strip_prefetch_on_ads: true,
            debug: false,
            max_wait_secs: 30,
            backup_seamless: false,
        }
    }

    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn with_max_wait(mut self, secs: u64) -> Self {
        self.max_wait_secs = secs;
        self
    }

    pub fn with_backup_seamless(mut self, enabled: bool) -> Self {
        self.backup_seamless = enabled;
        self
    }

    /// Зарезервировано под opt-in seamless backup. Default: None → остаёмся на strip.
    pub fn backup_playlist_hint(&self, _meta: &RequestMeta) -> Option<String> {
        if !self.backup_seamless {
            return None;
        }
        // Production GraphQL/token path — отдельный follow-up; сейчас только маркер.
        None
    }
}

fn host_matches(host: &str) -> bool {
    let h = host.to_ascii_lowercase();
    h.contains("ttvnw.net")
        || h.contains("twitch.tv")
        || h.contains("video-weaver")
        || h.contains("video-edge")
        || h.contains("playlist.ttvnw")
}

#[async_trait]
impl Plugin for TwitchPlugin {
    fn name(&self) -> &str {
        "twitch"
    }

    fn match_request(&self, req: &RequestMeta) -> bool {
        req.kind == ManifestKind::Hls && req.is_manifest && host_matches(&req.host)
    }

    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities {
            filter_segments: true,
            rewrite_urls: true,
            master_rewrite: true,
            filter_dash: false,
        }
    }

    fn filter_segments(
        &self,
        manifest: &mut Manifest,
        meta: &RequestMeta,
    ) -> Result<ProcessOutcome, PluginError> {
        self.playlists.fetch_add(1, Ordering::Relaxed);
        if manifest.kind() != PlaylistKind::Media {
            return Ok(ProcessOutcome::default());
        }

        let before = ose_detector::detect(manifest, &self.rules);
        let wait_est = before
            .ad_extinf_indices
            .iter()
            .filter_map(|&idx| {
                if let Some(Entry::Tag(Tag::ExtInf(ExtInf { duration, .. }))) =
                    manifest.entries.get(idx)
                {
                    Some(*duration)
                } else {
                    None
                }
            })
            .sum::<f64>()
            .ceil() as u64;

        let outcome = strip_ad_segments(manifest, &self.rules, self.strip_prefetch_on_ads)?;
        if outcome.ads_found {
            self.ads_found.fetch_add(1, Ordering::Relaxed);
            self.segments_removed
                .fetch_add(outcome.segments_removed, Ordering::Relaxed);
            if self.backup_seamless {
                if let Some(hint) = self.backup_playlist_hint(meta) {
                    debug!(%hint, "twitch backup_seamless hint (unused in strip mode)");
                } else if self.debug {
                    debug!("twitch backup_seamless enabled but no backup URL available — strip-only");
                }
            }
            if self.debug || outcome.ads_found {
                debug!(
                    host = %meta.host,
                    path = %meta.path,
                    segments_removed = outcome.segments_removed,
                    wait_est_secs = wait_est.min(self.max_wait_secs),
                    max_wait_secs = self.max_wait_secs,
                    "twitch ad stripped"
                );
            }
        }
        Ok(outcome)
    }

    fn rewrite_urls(
        &self,
        manifest: &mut Manifest,
        meta: &RequestMeta,
    ) -> Result<(), PluginError> {
        rewrite_master_variant_urls(manifest, meta)
    }

    fn stats(&self) -> PluginStats {
        PluginStats {
            playlists: self.playlists.load(Ordering::Relaxed),
            ads_found: self.ads_found.load(Ordering::Relaxed),
            segments_removed: self.segments_removed.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ose_manifest::{parse, serialize};

    #[tokio::test]
    async fn strip_midroll_inserts_discontinuity() {
        let raw = include_str!("../fixtures/midroll.m3u8");
        let plugin = TwitchPlugin::default();
        let m = parse(raw).unwrap();
        let meta = RequestMeta {
            host: "video-weaver.ttvnw.net".into(),
            path: "/v1/playlist/x.m3u8".into(),
            url: "https://video-weaver.ttvnw.net/v1/playlist/x.m3u8".into(),
            is_manifest: true,
            kind: ManifestKind::Hls,
            proxy_base: None,
        };
        let (out, outcome) = plugin.process_manifest(m, &meta).await.unwrap();
        assert_eq!(outcome.segments_removed, 3);
        let body = serialize(&out);
        assert!(!body.contains("ad201"));
        assert!(body.contains("seg101.ts"));
        assert!(body.contains("seg103.ts"));
        assert!(!body.contains("stitched"));
        assert!(body.contains("#EXT-X-DISCONTINUITY"));
    }

    #[tokio::test]
    async fn strip_prefetch_during_ads() {
        let raw = r#"#EXTM3U
#EXT-X-MEDIA-SEQUENCE:1
#EXT-X-DATERANGE:CLASS="twitch-stitched-ad"
#EXTINF:2.000,
ad1.ts
#EXT-X-TWITCH-PREFETCH:prefetch-ad.ts
#EXTINF:2.000,live
live1.ts
"#;
        let plugin = TwitchPlugin::default();
        let m = parse(raw).unwrap();
        let meta = RequestMeta {
            host: "ttvnw.net".into(),
            path: "/a.m3u8".into(),
            url: "https://ttvnw.net/a.m3u8".into(),
            is_manifest: true,
            kind: ManifestKind::Hls,
            proxy_base: None,
        };
        let (out, _) = plugin.process_manifest(m, &meta).await.unwrap();
        let body = serialize(&out);
        assert!(!body.contains("PREFETCH"));
        assert!(body.contains("live1.ts"));
    }
}
