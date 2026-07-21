//! Общий Segment Stripping для media playlist.

use std::collections::HashSet;

use ose_detector::{detect, is_ad_daterange, Rule};
use ose_manifest::{Entry, Manifest, PlaylistKind, Tag};

use crate::{PluginError, ProcessOutcome};

/// Удаляет рекламные сегменты по правилам detector и вставляет DISCONTINUITY.
pub fn strip_ad_segments(
    manifest: &mut Manifest,
    rules: &[Rule],
    strip_prefetch: bool,
) -> Result<ProcessOutcome, PluginError> {
    if manifest.kind() != PlaylistKind::Media {
        return Ok(ProcessOutcome::default());
    }

    let detection = detect(manifest, rules);
    if detection.ad_extinf_indices.is_empty() {
        return Ok(ProcessOutcome::default());
    }

    let ad_set: HashSet<usize> = detection.ad_extinf_indices.iter().copied().collect();
    let pairs = manifest.segment_pairs();

    let mut remove: HashSet<usize> = HashSet::new();
    let mut removed_segments = 0u64;
    let mut first_kept_segment_ordinal: Option<u64> = None;
    let mut need_disc_before: HashSet<usize> = HashSet::new();
    let mut prev_was_ad = false;

    let base_seq = manifest.media_sequence().unwrap_or(0);

    for (ordinal, (ext_i, uri_i)) in pairs.iter().enumerate() {
        let ordinal = ordinal as u64;
        if ad_set.contains(ext_i) {
            remove.insert(*ext_i);
            remove.insert(*uri_i);
            for k in *ext_i..*uri_i {
                if matches!(
                    &manifest.entries[k],
                    Entry::Tag(Tag::ProgramDateTime(_)) | Entry::Tag(Tag::Discontinuity)
                ) {
                    remove.insert(k);
                }
            }
            removed_segments += 1;
            prev_was_ad = true;
        } else {
            if prev_was_ad {
                need_disc_before.insert(*ext_i);
            }
            prev_was_ad = false;
            if first_kept_segment_ordinal.is_none() {
                first_kept_segment_ordinal = Some(ordinal);
            }
        }
    }

    for (idx, entry) in manifest.entries.iter().enumerate() {
        if let Entry::Tag(Tag::DateRange(attrs)) = entry {
            if is_ad_daterange(attrs) {
                remove.insert(idx);
            }
        }
        if strip_prefetch
            && removed_segments > 0
            && matches!(
                entry,
                Entry::Tag(Tag::TwitchPrefetch(_)) | Entry::Tag(Tag::Prefetch(_))
            )
        {
            remove.insert(idx);
        }
    }

    let mut new_entries = Vec::with_capacity(manifest.entries.len());
    for (i, e) in manifest.entries.iter().enumerate() {
        if remove.contains(&i) {
            continue;
        }
        if need_disc_before.contains(&i) {
            let already = new_entries
                .last()
                .is_some_and(|x| matches!(x, Entry::Tag(Tag::Discontinuity)));
            if !already {
                new_entries.push(Entry::Tag(Tag::Discontinuity));
            }
        }
        new_entries.push(e.clone());
    }
    manifest.entries = new_entries;

    if let Some(ord) = first_kept_segment_ordinal {
        manifest.set_media_sequence(base_seq + ord);
    }

    Ok(ProcessOutcome {
        ads_found: removed_segments > 0,
        segments_removed: removed_segments,
    })
}
