//! HTTP API: status, events, OpenMetrics.

use ose_neighbors::{coexistence_ok, detect_neighbors, Neighbor};
use ose_observe::{openmetrics_text, EngineEvent, EventRing};
use ose_plugin::PluginStats;
use parking_lot::Mutex;
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct RuntimeStats {
    pub active_streams: u64,
    pub plugin: PluginStats,
}

#[derive(Clone)]
pub struct StatusHandle {
    inner: Arc<Mutex<RuntimeStats>>,
    events: Arc<EventRing>,
}

impl StatusHandle {
    pub fn new() -> Self {
        Self::with_event_capacity(128)
    }

    pub fn with_event_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RuntimeStats::default())),
            events: Arc::new(EventRing::new(capacity)),
        }
    }

    pub fn events(&self) -> &EventRing {
        &self.events
    }

    pub fn push_event(&self, event: EngineEvent) {
        self.events.push(event);
    }

    pub fn set_plugin_stats(&self, plugin: PluginStats) {
        self.inner.lock().plugin = plugin;
    }

    pub fn set_active_streams(&self, n: u64) {
        self.inner.lock().active_streams = n;
    }

    pub fn bump_streams(&self, delta: i64) {
        let mut g = self.inner.lock();
        if delta >= 0 {
            g.active_streams = g.active_streams.saturating_add(delta as u64);
        } else {
            g.active_streams = g.active_streams.saturating_sub((-delta) as u64);
        }
    }

    pub fn snapshot(&self, plugins: &[String]) -> StatusResponse {
        let g = self.inner.lock();
        let neighbors = detect_neighbors();
        let ok = coexistence_ok(&neighbors);
        StatusResponse {
            streams: g.active_streams,
            ads: g.plugin.ads_found,
            removed_segments: g.plugin.segments_removed,
            playlists: g.plugin.playlists,
            plugins: plugins.to_vec(),
            neighbors,
            coexistence_ok: ok,
            mode_hint: if ok {
                "transparent_ok"
            } else {
                "prefer_transparent_check_tpws"
            }
            .into(),
            events_buffered: self.events.len() as u64,
            plugin_abi: ose_plugin::PLUGIN_ABI_VERSION,
        }
    }

    pub fn metrics_text(&self, coalesce_inflight: u64) -> String {
        let g = self.inner.lock();
        openmetrics_text(
            g.plugin.playlists,
            g.plugin.ads_found,
            g.plugin.segments_removed,
            g.active_streams,
            self.events.len() as u64,
            coalesce_inflight,
        )
    }
}

impl Default for StatusHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub streams: u64,
    pub ads: u64,
    pub removed_segments: u64,
    pub playlists: u64,
    pub plugins: Vec<String>,
    pub neighbors: Vec<Neighbor>,
    pub coexistence_ok: bool,
    pub mode_hint: String,
    pub events_buffered: u64,
    pub plugin_abi: u32,
}

pub fn status_json(handle: &StatusHandle, plugins: &[String]) -> String {
    serde_json::to_string_pretty(&handle.snapshot(plugins)).unwrap_or_else(|_| "{}".into())
}

pub fn events_json(handle: &StatusHandle) -> String {
    serde_json::to_string_pretty(&handle.events().snapshot()).unwrap_or_else(|_| "[]".into())
}
