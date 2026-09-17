//! Transcript cue normalization.

use super::SubtitleCue;

pub(super) fn sanitize(mut cues: Vec<SubtitleCue>, duration_ms: u64) -> Vec<SubtitleCue> {
    cues.sort_by_key(|cue| (cue.start_ms, cue.end_ms));
    let mut previous_start = 0_u64;
    cues.into_iter()
        .filter_map(|cue| {
            let text = cue.text.trim().to_owned();
            let start_ms = cue.start_ms.max(previous_start);
            let end_ms = cue.end_ms.min(duration_ms);
            if text.is_empty() || start_ms >= duration_ms || end_ms <= start_ms {
                return None;
            }
            previous_start = start_ms;
            Some(SubtitleCue {
                start_ms,
                end_ms,
                text,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
