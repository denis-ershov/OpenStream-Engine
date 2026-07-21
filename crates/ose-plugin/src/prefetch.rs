//! Политика LL-HLS prefetch на уровне ядра.

use ose_manifest::{Entry, Manifest, Tag};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrefetchPolicy {
    /// Не трогать prefetch-теги.
    #[default]
    Keep,
    /// Удалить все `#EXT-X-PREFETCH` / `#EXT-X-TWITCH-PREFETCH`.
    StripAll,
    /// Удалить prefetch только если в плейлисте уже вырезались ads (caller передаёт флаг).
    StripWhenAdsRemoved,
}

/// Применяет политику prefetch к манифесту.
pub fn apply_prefetch_policy(
    manifest: &mut Manifest,
    policy: PrefetchPolicy,
    ads_were_removed: bool,
) {
    let strip = match policy {
        PrefetchPolicy::Keep => false,
        PrefetchPolicy::StripAll => true,
        PrefetchPolicy::StripWhenAdsRemoved => ads_were_removed,
    };
    if !strip {
        return;
    }
    manifest.entries.retain(|e| {
        !matches!(
            e,
            Entry::Tag(Tag::Prefetch(_)) | Entry::Tag(Tag::TwitchPrefetch(_))
        )
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use ose_manifest::parse;

    #[test]
    fn strip_all_prefetch() {
        let raw = r#"#EXTM3U
#EXTINF:2.000,live
a.ts
#EXT-X-TWITCH-PREFETCH:b.ts
"#;
        let mut m = parse(raw).unwrap();
        apply_prefetch_policy(&mut m, PrefetchPolicy::StripAll, false);
        let s = m.to_string();
        assert!(!s.contains("PREFETCH"));
    }
}
