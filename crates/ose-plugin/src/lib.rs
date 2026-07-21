//! Plugin API и менеджер плагинов.

use std::sync::Arc;

use async_trait::async_trait;
use ose_dash::Mpd;
use ose_manifest::Manifest;
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod prefetch;
mod rewrite;
mod strip;

pub use ose_media::{FilterOutcome, ManifestKind, MediaError, MediaFilter};
pub use prefetch::{apply_prefetch_policy, PrefetchPolicy};
pub use rewrite::rewrite_master_variant_urls;
pub use strip::strip_ad_segments;

/// Версия Plugin ABI (см. docs/adr/0001-plugin-abi.md). Breaking change trait → bump.
pub const PLUGIN_ABI_VERSION: u32 = 3;

#[derive(Debug, Clone)]
pub struct RequestMeta {
    pub host: String,
    pub path: String,
    pub url: String,
    pub is_manifest: bool,
    pub kind: ManifestKind,
    /// Базовый URL прокси для rewrite master (например `http://192.168.1.1:18080`).
    pub proxy_base: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginStats {
    pub playlists: u64,
    pub ads_found: u64,
    pub segments_removed: u64,
}

#[derive(Debug, Clone, Default)]
pub struct PluginCapabilities {
    pub filter_segments: bool,
    pub rewrite_urls: bool,
    pub master_rewrite: bool,
    pub filter_dash: bool,
}

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("{0}")]
    Msg(String),
}

#[derive(Debug, Clone, Default)]
pub struct ProcessOutcome {
    pub ads_found: bool,
    pub segments_removed: u64,
}

/// Плагин сервиса. HLS: `filter_segments` → `rewrite_urls`. DASH: `filter_dash`.
#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn match_request(&self, req: &RequestMeta) -> bool;

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
        _manifest: &mut Manifest,
        _meta: &RequestMeta,
    ) -> Result<ProcessOutcome, PluginError> {
        Ok(ProcessOutcome::default())
    }

    fn rewrite_urls(
        &self,
        _manifest: &mut Manifest,
        _meta: &RequestMeta,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    fn filter_dash(
        &self,
        _mpd: &mut Mpd,
        _meta: &RequestMeta,
    ) -> Result<ProcessOutcome, PluginError> {
        Ok(ProcessOutcome::default())
    }

    async fn process_manifest(
        &self,
        mut manifest: Manifest,
        meta: &RequestMeta,
    ) -> Result<(Manifest, ProcessOutcome), PluginError> {
        let outcome = self.filter_segments(&mut manifest, meta)?;
        self.rewrite_urls(&mut manifest, meta)?;
        Ok((manifest, outcome))
    }

    async fn process_mpd(
        &self,
        mut mpd: Mpd,
        meta: &RequestMeta,
    ) -> Result<(Mpd, ProcessOutcome), PluginError> {
        let outcome = self.filter_dash(&mut mpd, meta)?;
        Ok((mpd, outcome))
    }

    fn stats(&self) -> PluginStats;
}

pub struct PluginManager {
    plugins: Vec<Arc<dyn Plugin>>,
}

impl PluginManager {
    pub fn new(plugins: Vec<Arc<dyn Plugin>>) -> Self {
        Self { plugins }
    }

    pub fn find(&self, req: &RequestMeta) -> Option<&dyn Plugin> {
        self.plugins
            .iter()
            .find(|p| p.match_request(req))
            .map(|p| p.as_ref())
    }

    pub fn find_arc(&self, req: &RequestMeta) -> Option<Arc<dyn Plugin>> {
        self.plugins
            .iter()
            .find(|p| p.match_request(req))
            .cloned()
    }

    pub fn aggregate_stats(&self) -> PluginStats {
        let mut total = PluginStats::default();
        for p in &self.plugins {
            let s = p.stats();
            total.playlists += s.playlists;
            total.ads_found += s.ads_found;
            total.segments_removed += s.segments_removed;
        }
        total
    }

    pub fn plugin_names(&self) -> Vec<String> {
        self.plugins.iter().map(|p| p.name().to_string()).collect()
    }
}
