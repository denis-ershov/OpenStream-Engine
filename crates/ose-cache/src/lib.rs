//! Cache Engine: TTL-кэш манифестов. Ключ = identity (URL + optional etag/body hash).

use std::collections::HashMap;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
    /// Короткий отпечаток тела (например FNV/xxhash hex), если etag нет.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_hash: Option<String>,
}

impl CacheKey {
    pub fn from_url(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            etag: None,
            body_hash: None,
        }
    }

    pub fn with_etag(mut self, etag: impl Into<String>) -> Self {
        self.etag = Some(etag.into());
        self
    }

    pub fn with_body_hash(mut self, hash: impl Into<String>) -> Self {
        self.body_hash = Some(hash.into());
        self
    }

    /// Стабильная строка для хранения / логов.
    pub fn identity(&self) -> String {
        match (&self.etag, &self.body_hash) {
            (Some(e), _) => format!("{}|etag:{}", self.url, e),
            (None, Some(h)) => format!("{}|hash:{}", self.url, h),
            (None, None) => self.url.clone(),
        }
    }
}

/// Простой FNV-1a 64 для body fingerprint без тяжёлых зависимостей.
pub fn fnv1a64(data: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for b in data {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

pub fn body_hash_hex(data: &[u8]) -> String {
    format!("{:016x}", fnv1a64(data))
}

#[derive(Clone)]
struct Entry {
    body: String,
    media_sequence: Option<u64>,
    inserted: Instant,
}

pub struct PlaylistCache {
    ttl: Duration,
    max_entries: usize,
    inner: Mutex<HashMap<String, Entry>>,
}

impl PlaylistCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl: ttl.clamp(Duration::from_secs(1), Duration::from_secs(5)),
            max_entries: 256,
            inner: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<(String, Option<u64>)> {
        self.get_key(&CacheKey::from_url(key))
    }

    pub fn get_key(&self, key: &CacheKey) -> Option<(String, Option<u64>)> {
        let id = key.identity();
        let mut map = self.inner.lock();
        let expired = map
            .get(&id)
            .map(|e| e.inserted.elapsed() > self.ttl)
            .unwrap_or(true);
        if expired {
            map.remove(&id);
            return None;
        }
        map.get(&id).map(|e| (e.body.clone(), e.media_sequence))
    }

    pub fn put(&self, key: &str, body: String, media_sequence: Option<u64>) {
        self.put_key(&CacheKey::from_url(key), body, media_sequence);
    }

    pub fn put_key(&self, key: &CacheKey, body: String, media_sequence: Option<u64>) {
        let id = key.identity();
        let mut map = self.inner.lock();
        if map.len() >= self.max_entries {
            let ttl = self.ttl;
            map.retain(|_, e| e.inserted.elapsed() <= ttl);
            if map.len() >= self.max_entries {
                map.clear();
            }
        }
        map.insert(
            id,
            Entry {
                body,
                media_sequence,
                inserted: Instant::now(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ttl_roundtrip() {
        let c = PlaylistCache::new(Duration::from_secs(2));
        c.put("https://example/a.m3u8", "body".into(), Some(1));
        let (b, s) = c.get("https://example/a.m3u8").unwrap();
        assert_eq!(b, "body");
        assert_eq!(s, Some(1));
        assert!(c.get("https://other").is_none());
    }

    #[test]
    fn keyspace_etag_and_hash() {
        let c = PlaylistCache::new(Duration::from_secs(2));
        let k1 = CacheKey::from_url("https://cdn/x.mpd").with_etag("\"abc\"");
        let k2 = CacheKey::from_url("https://cdn/x.mpd").with_body_hash(body_hash_hex(b"v1"));
        c.put_key(&k1, "mpd1".into(), None);
        c.put_key(&k2, "mpd2".into(), None);
        assert_eq!(c.get_key(&k1).unwrap().0, "mpd1");
        assert_eq!(c.get_key(&k2).unwrap().0, "mpd2");
        assert_ne!(k1.identity(), k2.identity());
    }
}
