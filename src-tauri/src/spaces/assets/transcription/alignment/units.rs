use sha2::{Digest, Sha256};
use std::ops::Range;

use crate::{
    ai::contracts::speech_wire::{AlignedSpeechUnit, AlignmentUnitKind},
    media::transcript::TranscriptUnit,
};

pub(super) fn map_units(
    segment_id: &str,
    source: &str,
    segment_start_ms: u64,
    segment_end_ms: u64,
    aligned: Vec<AlignedSpeechUnit>,
) -> Result<Vec<TranscriptUnit>, String> {
    let characters = source.chars().collect::<Vec<_>>();
    let mut cursor = 0_usize;
    let mut output = Vec::new();
    for unit in aligned {
        let needle = unit.text.trim();
        if needle.is_empty() {
            continue;
        }
        let remaining = characters[cursor..].iter().collect::<String>();
        let Some(byte_offset) = remaining.find(needle) else {
            continue;
        };
        let relative_start = remaining[..byte_offset].chars().count();
        let start = cursor + relative_start;
        let end = start + needle.chars().count();
        if start > cursor {
            output.push(source_unit(
                segment_id,
                &characters,
                cursor..start,
                (None, None),
                None,
                Some("not_aligned"),
                "segment",
            )?);
        }
        let (start_ms, end_ms, reason) = match (unit.start_ms, unit.end_ms) {
            (Some(start_ms), Some(end_ms))
                if start_ms >= segment_start_ms
                    && end_ms <= segment_end_ms
                    && end_ms > start_ms =>
            {
                (
                    Some(start_ms),
                    Some(end_ms),
                    unit.unaligned_reason.as_deref(),
                )
            }
            _ => (None, None, Some("invalid_or_missing_timing")),
        };
        let kind = match unit.unit_kind {
            AlignmentUnitKind::Word => "word",
            AlignmentUnitKind::Character => "character",
        };
        output.push(source_unit(
            segment_id,
            &characters,
            start..end,
            (start_ms, end_ms),
            unit.score,
            reason,
            kind,
        )?);
        cursor = end;
    }
    if cursor < characters.len() {
        output.push(source_unit(
            segment_id,
            &characters,
            cursor..characters.len(),
            (None, None),
            None,
            Some("not_aligned"),
            "segment",
        )?);
    }
    Ok(output)
}

fn source_unit(
    segment_id: &str,
    source: &[char],
    range: Range<usize>,
    timing: (Option<u64>, Option<u64>),
    score: Option<f32>,
    reason: Option<&str>,
    kind: &str,
) -> Result<TranscriptUnit, String> {
    let start_character = u32::try_from(range.start).map_err(|error| error.to_string())?;
    let end_character = u32::try_from(range.end).map_err(|error| error.to_string())?;
    let mut digest = Sha256::new();
    digest.update(segment_id.as_bytes());
    digest.update([0]);
    digest.update(start_character.to_be_bytes());
    digest.update(end_character.to_be_bytes());
    Ok(TranscriptUnit {
        id: format!("{:x}", digest.finalize()),
        text: source[range].iter().collect(),
        start_character,
        end_character,
        start_ms: timing.0,
        end_ms: timing.1,
        score,
        unaligned_reason: reason.map(str::to_owned),
        unit_kind: kind.into(),
    })
}
