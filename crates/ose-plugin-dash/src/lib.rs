//! Plugin DASH: фильтрация рекламных Period / AdaptationSet.

use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use ose_dash::{filter_ad_nodes, DashFilterRules, Mpd};
use ose_media::{FilterOutcome, ManifestKind, MediaError, MediaFilter};
use ose_plugin::{
    Plugin, PluginCapabilities, PluginError, PluginStats, ProcessOutcome, RequestMeta,
};
use tracing::debug;

pub struct DashPlugin {
    rules: DashFilterRules,
    hosts: Vec<String>,
    playlists: AtomicU64,
    ads_found: AtomicU64,
    units_removed: AtomicU64,
    pub debug: bool,
}

impl Default for DashPlugin {
    fn default() -> Self {
        Self::new(DashFilterRules::default(), Vec::new())
    }
}

impl DashPlugin {
    pub fn new(rules: DashFilterRules, hosts: Vec<String>) -> Self {
        Self {
            rules,
            hosts,
            playlists: AtomicU64::new(0),
            ads_found: AtomicU64::new(0),
            units_removed: AtomicU64::new(0),
            debug: false,
        }
    }

    /// Принимает любые `.mpd` (hosts пустой = match по kind).
    pub fn universal() -> Self {
        Self::new(DashFilterRules::default(), Vec::new())
    }

    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn with_hosts(mut self, hosts: Vec<String>) -> Self {
        self.hosts = hosts;
        self
    }

    fn host_ok(&self, host: &str) -> bool {
        if self.hosts.is_empty() {
            return true;
        }
        let h = host.to_ascii_lowercase();
        self.hosts.iter().any(|p| {
            let p = p.to_ascii_lowercase();
            !p.is_empty() && h.contains(&p)
        })
    }
}

impl MediaFilter for DashPlugin {
    fn name(&self) -> &str {
        "dash"
    }

    fn apply_dash(&self, mpd: &mut Mpd) -> Result<FilterOutcome, MediaError> {
        let stats = filter_ad_nodes(mpd, &self.rules);
        Ok(FilterOutcome {
            ads_found: stats.any_removed(),
            units_removed: stats.periods_removed + stats.adaptations_removed,
        })
    }
}

#[async_trait]
impl Plugin for DashPlugin {
    fn name(&self) -> &str {
        "dash"
    }

    fn match_request(&self, req: &RequestMeta) -> bool {
        req.kind == ManifestKind::Dash && self.host_ok(&req.host)
    }

    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities {
            filter_segments: true,
            rewrite_urls: false,
            master_rewrite: false,
            filter_dash: true,
        }
    }

    fn filter_dash(
        &self,
        mpd: &mut Mpd,
        meta: &RequestMeta,
    ) -> Result<ProcessOutcome, PluginError> {
        self.playlists.fetch_add(1, Ordering::Relaxed);
        let outcome =
            self.apply_dash(mpd)
                .map_err(|e| PluginError::Msg(e.to_string()))?;
        if outcome.ads_found {
            self.ads_found.fetch_add(1, Ordering::Relaxed);
            self.units_removed
                .fetch_add(outcome.units_removed, Ordering::Relaxed);
            if self.debug {
                debug!(
                    host = %meta.host,
                    removed = outcome.units_removed,
                    "dash ad nodes stripped"
                );
            }
        }
        Ok(ProcessOutcome {
            ads_found: outcome.ads_found,
            segments_removed: outcome.units_removed,
        })
    }

    async fn process_mpd(
        &self,
        mut mpd: Mpd,
        meta: &RequestMeta,
    ) -> Result<(Mpd, ProcessOutcome), PluginError> {
        let outcome = Plugin::filter_dash(self, &mut mpd, meta)?;
        Ok((mpd, outcome))
    }

    fn stats(&self) -> PluginStats {
        PluginStats {
            playlists: self.playlists.load(Ordering::Relaxed),
            ads_found: self.ads_found.load(Ordering::Relaxed),
            segments_removed: self.units_removed.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn process_ad_period() {
        let xml = r#"<MPD type="static">
  <Period id="live"><AdaptationSet contentType="video"><Representation id="v" bandwidth="1"/></AdaptationSet></Period>
  <Period id="ad_break"><AssetIdentifier schemeIdUri="urn:scte:dash:ad" value="x"/></Period>
</MPD>"#;
        let plugin = DashPlugin::universal();
        let meta = RequestMeta {
            host: "cdn.example".into(),
            path: "/manifest.mpd".into(),
            url: "https://cdn.example/manifest.mpd".into(),
            is_manifest: true,
            kind: ManifestKind::Dash,
            proxy_base: None,
        };
        assert!(plugin.match_request(&meta));
        let mpd = Mpd::parse(xml).unwrap();
        let (out, outcome) = plugin.process_mpd(mpd, &meta).await.unwrap();
        assert!(outcome.ads_found);
        assert_eq!(out.period_count(), 1);
    }
}
