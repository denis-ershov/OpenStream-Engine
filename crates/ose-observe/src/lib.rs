//! Observability: ring-buffer событий для LuCI + OpenMetrics text.

use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    AdsStripped,
    ManifestProcessed,
    Reload,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineEvent {
    pub ts_unix_ms: u64,
    pub kind: EventKind,
    pub plugin: String,
    pub host: String,
    pub path: String,
    pub detail: String,
    pub units_removed: u64,
}

impl EngineEvent {
    pub fn now(
        kind: EventKind,
        plugin: impl Into<String>,
        host: impl Into<String>,
        path: impl Into<String>,
        detail: impl Into<String>,
        units_removed: u64,
    ) -> Self {
        let ts_unix_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Self {
            ts_unix_ms,
            kind,
            plugin: plugin.into(),
            host: host.into(),
            path: path.into(),
            detail: detail.into(),
            units_removed,
        }
    }
}

/// Фиксированный ring-buffer (без роста RSS).
pub struct EventRing {
    capacity: usize,
    inner: Mutex<VecDeque<EngineEvent>>,
}

impl EventRing {
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            capacity,
            inner: Mutex::new(VecDeque::with_capacity(capacity)),
        }
    }

    pub fn push(&self, event: EngineEvent) {
        let mut q = self.inner.lock();
        if q.len() >= self.capacity {
            q.pop_front();
        }
        q.push_back(event);
    }

    pub fn snapshot(&self) -> Vec<EngineEvent> {
        self.inner.lock().iter().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.inner.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.lock().is_empty()
    }

    pub fn clear(&self) {
        self.inner.lock().clear();
    }
}

/// Минимальный OpenMetrics/Prometheus text exposition (без тяжёлых crate).
pub fn openmetrics_text(
    playlists: u64,
    ads_found: u64,
    segments_removed: u64,
    active_streams: u64,
    events_buffered: u64,
    coalesced_inflight: u64,
) -> String {
    let mut out = String::with_capacity(512);
    out.push_str("# HELP openstream_playlists_total Manifests processed\n");
    out.push_str("# TYPE openstream_playlists_total counter\n");
    out.push_str(&format!("openstream_playlists_total {playlists}\n"));
    out.push_str("# HELP openstream_ads_found_total Playlists with ads detected\n");
    out.push_str("# TYPE openstream_ads_found_total counter\n");
    out.push_str(&format!("openstream_ads_found_total {ads_found}\n"));
    out.push_str("# HELP openstream_segments_removed_total HLS segments / DASH nodes removed\n");
    out.push_str("# TYPE openstream_segments_removed_total counter\n");
    out.push_str(&format!("openstream_segments_removed_total {segments_removed}\n"));
    out.push_str("# HELP openstream_active_streams Current proxy connections\n");
    out.push_str("# TYPE openstream_active_streams gauge\n");
    out.push_str(&format!("openstream_active_streams {active_streams}\n"));
    out.push_str("# HELP openstream_events_buffered Events in ring buffer\n");
    out.push_str("# TYPE openstream_events_buffered gauge\n");
    out.push_str(&format!("openstream_events_buffered {events_buffered}\n"));
    out.push_str("# HELP openstream_coalesce_inflight In-flight coalesced computations\n");
    out.push_str("# TYPE openstream_coalesce_inflight gauge\n");
    out.push_str(&format!("openstream_coalesce_inflight {coalesced_inflight}\n"));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_drops_oldest() {
        let ring = EventRing::new(3);
        for i in 0..5 {
            ring.push(EngineEvent::now(
                EventKind::AdsStripped,
                "twitch",
                "h",
                "/p",
                format!("n={i}"),
                i,
            ));
        }
        assert_eq!(ring.len(), 3);
        let snap = ring.snapshot();
        assert_eq!(snap[0].units_removed, 2);
        assert_eq!(snap[2].units_removed, 4);
    }

    #[test]
    fn metrics_contains_counters() {
        let t = openmetrics_text(1, 2, 3, 4, 5, 0);
        assert!(t.contains("openstream_playlists_total 1"));
        assert!(t.contains("openstream_active_streams 4"));
    }
}
