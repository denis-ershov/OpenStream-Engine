//! Generic HLS plugin на базе RuleSet + пресеты Kick/Trovo.

use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use ose_detector::Rule;
use ose_manifest::{Manifest, PlaylistKind};
use ose_plugin::{
    rewrite_master_variant_urls, strip_ad_segments, ManifestKind, Plugin, PluginCapabilities,
    PluginError, PluginStats, ProcessOutcome, RequestMeta,
};
use ose_rules::{host_matches, kick_default, trovo_default, youtube_default, RuleSet};
use tracing::debug;

pub struct RulesHlsPlugin {
    ruleset: RuleSet,
    rules: Vec<Rule>,
    playlists: AtomicU64,
    ads_found: AtomicU64,
    segments_removed: AtomicU64,
    pub debug: bool,
}

impl RulesHlsPlugin {
    pub fn from_ruleset(ruleset: RuleSet) -> Self {
        let rules: Vec<Rule> = ruleset.rules.iter().cloned().map(|r| r.into_rule()).collect();
        Self {
            ruleset,
            rules,
            playlists: AtomicU64::new(0),
            ads_found: AtomicU64::new(0),
            segments_removed: AtomicU64::new(0),
            debug: false,
        }
    }

    pub fn kick() -> Self {
        Self::from_ruleset(kick_default())
    }

    pub fn trovo() -> Self {
        Self::from_ruleset(trovo_default())
    }

    pub fn youtube() -> Self {
        Self::from_ruleset(youtube_default())
    }

    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
}

#[async_trait]
impl Plugin for RulesHlsPlugin {
    fn name(&self) -> &str {
        &self.ruleset.name
    }

    fn match_request(&self, req: &RequestMeta) -> bool {
        self.ruleset.enabled
            && req.kind == ManifestKind::Hls
            && req.is_manifest
            && host_matches(&req.host, &self.ruleset.hosts)
    }

    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities {
            filter_segments: true,
            rewrite_urls: self.ruleset.rewrite_master,
            master_rewrite: self.ruleset.rewrite_master,
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
        let outcome = strip_ad_segments(manifest, &self.rules, self.ruleset.strip_prefetch_on_ads)?;
        if outcome.ads_found {
            self.ads_found.fetch_add(1, Ordering::Relaxed);
            self.segments_removed
                .fetch_add(outcome.segments_removed, Ordering::Relaxed);
            if self.debug {
                debug!(
                    plugin = %self.ruleset.name,
                    host = %meta.host,
                    removed = outcome.segments_removed,
                    "hls rules strip"
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
        if self.ruleset.rewrite_master {
            rewrite_master_variant_urls(manifest, meta)?;
        }
        Ok(())
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
    use ose_manifest::parse;

    #[tokio::test]
    async fn kick_match_and_strip() {
        let plugin = RulesHlsPlugin::kick();
        let meta = RequestMeta {
            host: "stream.kick.com".into(),
            path: "/x.m3u8".into(),
            url: "https://stream.kick.com/x.m3u8".into(),
            is_manifest: true,
            kind: ManifestKind::Hls,
            proxy_base: None,
        };
        assert!(plugin.match_request(&meta));
        let raw = r#"#EXTM3U
#EXT-X-MEDIA-SEQUENCE:1
#EXT-X-DATERANGE:ID="stitched-ad",CLASS="stitched"
#EXTINF:2.000,
ad.ts
#EXTINF:2.000,
live.ts
"#;
        let m = parse(raw).unwrap();
        let (out, outcome) = plugin.process_manifest(m, &meta).await.unwrap();
        assert!(outcome.segments_removed >= 1);
        assert!(!out.to_string().contains("ad.ts"));
    }

    #[tokio::test]
    async fn kick_fixture_midroll() {
        let plugin = RulesHlsPlugin::kick();
        let meta = RequestMeta {
            host: "fa000.kickusercontent.com".into(),
            path: "/playlist.m3u8".into(),
            url: "https://fa000.kickusercontent.com/playlist.m3u8".into(),
            is_manifest: true,
            kind: ManifestKind::Hls,
            proxy_base: None,
        };
        let raw = include_str!("../fixtures/kick_midroll.m3u8");
        let m = parse(raw).unwrap();
        let (out, outcome) = plugin.process_manifest(m, &meta).await.unwrap();
        assert!(outcome.segments_removed >= 1);
        assert!(!out.to_string().contains("ad_segment"));
        assert!(out.to_string().contains("live_b.ts"));
    }

    #[tokio::test]
    async fn youtube_fixture_ad_uri() {
        let plugin = RulesHlsPlugin::youtube();
        let meta = RequestMeta {
            host: "rr1---sn-xxx.googlevideo.com".into(),
            path: "/live.m3u8".into(),
            url: "https://rr1---sn-xxx.googlevideo.com/live.m3u8".into(),
            is_manifest: true,
            kind: ManifestKind::Hls,
            proxy_base: None,
        };
        assert!(plugin.match_request(&meta));
        let raw = include_str!("../fixtures/youtube_ad_uri.m3u8");
        let m = parse(raw).unwrap();
        let (out, outcome) = plugin.process_manifest(m, &meta).await.unwrap();
        assert!(outcome.segments_removed >= 1);
        assert!(!out.to_string().contains("ad_break"));
        assert!(out.to_string().contains("live_002.ts"));
    }
}
