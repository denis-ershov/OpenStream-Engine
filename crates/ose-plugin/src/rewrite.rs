//! Rewrite URL вариантов в master playlist через HTTP proxy.

use ose_manifest::{Entry, Manifest, PlaylistKind, Tag};

use crate::{PluginError, RequestMeta};

/// Переписывает URI после `#EXT-X-STREAM-INF` в absolute URL через proxy_base.
///
/// Пример: `https://cdn/low.m3u8` → `http://router:18080/https://cdn/low.m3u8`
/// (absolute-form для forward proxy) либо оставляем как есть, если proxy_base не задан.
pub fn rewrite_master_variant_urls(
    manifest: &mut Manifest,
    meta: &RequestMeta,
) -> Result<(), PluginError> {
    if manifest.kind() != PlaylistKind::Master {
        return Ok(());
    }
    let Some(base) = meta.proxy_base.as_deref() else {
        return Ok(());
    };
    let base = base.trim_end_matches('/');

    let mut after_stream_inf = false;
    for entry in &mut manifest.entries {
        match entry {
            Entry::Tag(Tag::StreamInf(_)) => {
                after_stream_inf = true;
            }
            Entry::Uri(uri) if after_stream_inf => {
                *uri = rewrite_one(base, uri, &meta.url);
                after_stream_inf = false;
            }
            Entry::Blank | Entry::Comment(_) => {}
            _ => {
                after_stream_inf = false;
            }
        }
    }
    Ok(())
}

fn rewrite_one(proxy_base: &str, uri: &str, playlist_url: &str) -> String {
    if uri.starts_with(proxy_base) {
        return uri.to_string();
    }
    let absolute = if uri.starts_with("http://") || uri.starts_with("https://") {
        uri.to_string()
    } else {
        resolve_relative(playlist_url, uri)
    };
    // Absolute-form proxy URL: GET http://proxy/https://origin/...
    format!("{proxy_base}/{absolute}")
}

fn resolve_relative(base_url: &str, rel: &str) -> String {
    if let Some(idx) = base_url.rfind('/') {
        format!("{}{}", &base_url[..=idx], rel.trim_start_matches('/'))
    } else {
        rel.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ose_manifest::parse;

    #[test]
    fn rewrite_master() {
        let raw = r#"#EXTM3U
#EXT-X-STREAM-INF:BANDWIDTH=1000
low.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=3000
https://cdn.example/high.m3u8
"#;
        let mut m = parse(raw).unwrap();
        let meta = RequestMeta {
            host: "cdn.example".into(),
            path: "/master.m3u8".into(),
            url: "https://cdn.example/master.m3u8".into(),
            is_manifest: true,
            kind: crate::ManifestKind::Hls,
            proxy_base: Some("http://192.168.1.1:18080".into()),
        };
        rewrite_master_variant_urls(&mut m, &meta).unwrap();
        let uris: Vec<_> = m
            .entries
            .iter()
            .filter_map(|e| match e {
                Entry::Uri(u) => Some(u.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            uris[0],
            "http://192.168.1.1:18080/https://cdn.example/low.m3u8"
        );
        assert_eq!(
            uris[1],
            "http://192.168.1.1:18080/https://cdn.example/high.m3u8"
        );
    }
}
