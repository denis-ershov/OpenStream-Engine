//! Skeleton: скопируйте в свой crate и подключите path-зависимости на ose-plugin.

// Раскомментируйте после добавления зависимостей:
/*
use async_trait::async_trait;
use ose_manifest::Manifest;
use ose_plugin::{
    Plugin, PluginCapabilities, PluginError, PluginStats, ProcessOutcome, RequestMeta,
};

pub struct ExamplePlugin {
    pub hosts: Vec<String>,
}

#[async_trait]
impl Plugin for ExamplePlugin {
    fn name(&self) -> &str {
        "example"
    }

    fn match_request(&self, req: &RequestMeta) -> bool {
        req.is_manifest
            && self
                .hosts
                .iter()
                .any(|h| req.host.to_ascii_lowercase().contains(&h.to_ascii_lowercase()))
    }

    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities {
            filter_segments: true,
            rewrite_urls: false,
            master_rewrite: false,
            filter_dash: false,
        }
    }

    fn filter_segments(
        &self,
        _manifest: &mut Manifest,
        _meta: &RequestMeta,
    ) -> Result<ProcessOutcome, PluginError> {
        // detect → strip через ose_plugin::strip_ad_segments
        Ok(ProcessOutcome::default())
    }

    fn stats(&self) -> PluginStats {
        PluginStats::default()
    }
}
*/

/// Placeholder, чтобы шаблон компилировался без workspace deps.
pub const SKELETON_NOTE: &str = "wire ose-plugin path deps and uncomment Plugin impl";
