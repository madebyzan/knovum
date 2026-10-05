use provenance::{validate_optional_provenance, validate_provenance, validate_speaker_tracks};
mod provenance;
use std::collections::HashSet;

use serde_json::{Map, Value};

use crate::media::transcript::{MediaTranscript, SubtitleCue, TranscriptUnit};

const MAX_TEXT: usize = 4 * 1024 * 1024;
const MAX_SEGMENTS: usize = 100_000;
const MAX_UNITS: usize = 16_000;
const MAX_CUES: usize = 100_000;
const MAX_SPEAKER_TRACKS: usize = 1_024;
const MAX_SPEAKER_TURNS: usize = 100_000;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

pub(super) struct TranscriptContract {
    pub(super) duration: i64,
    pub(super) starts: HashSet<i64>,
    pub(super) first_start: Option<i64>,
}

pub(super) fn validate(object: &Map<String, Value>) -> Result<TranscriptContract, String> {
    let value = object
        .get("transcript")
        .ok_or_else(|| "mediaKnowledge transcript is required".to_string())?;
    let transcript: MediaTranscript = serde_json::from_value(value.clone())
        .map_err(|error| format!("Media transcript is invalid: {error}"))?;
    let source_revision = object
        .get("sourceRevision")
        .and_then(Value::as_str)
        .ok_or("mediaKnowledge sourceRevision is required")?;
    validate_transcript(&transcript, source_revision)?;
    let duration = i64::try_from(transcript.duration_ms)
        .map_err(|_| "Transcript duration exceeds its supported range")?;
    let starts = transcript
        .segments
        .iter()
        .map(|segment| i64::try_from(segment.start_ms))
        .collect::<Result<HashSet<_>, _>>()
        .map_err(|_| "Transcript segment timing exceeds its supported range")?;
    let first_start = transcript
        .segments
        .first()
        .map(|segment| i64::try_from(segment.start_ms))
        .transpose()
        .map_err(|_| "Transcript segment timing exceeds its supported range")?;
    Ok(TranscriptContract {
        duration,
        starts,
        first_start,
    })
}

fn validate_transcript(
    transcript: &MediaTranscript,
    expected_source_revision: &str,
) -> Result<(), String> {
    require_sha256(&transcript.id, "Transcript id")?;
    if transcript.source_revision != expected_source_revision {
        return Err("Transcript sourceRevision must match mediaKnowledge sourceRevision".into());
    }
    require_bounded_text(
        &transcript.source_revision,
        128,
        "Transcript sourceRevision",
    )?;
    require_revision(transcript.revision, "Transcript revision")?;
    require_bounded_text(&transcript.language, 64, "Transcript language")?;
    require_safe_timestamp(transcript.duration_ms, "Transcript duration")?;
    if transcript.text.encode_utf16().count() > MAX_TEXT {
        return Err("Transcript text is too long".into());
    }
    validate_provenance(&transcript.provenance)?;
    validate_optional_provenance(transcript)?;
    let speaker_ids = validate_speaker_tracks(transcript)?;
    validate_segments(transcript, &speaker_ids)?;
    validate_cues(&transcript.subtitle_cues, transcript.duration_ms)?;
    let expected_text = transcript
        .segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    if transcript.text != expected_text {
        return Err("Transcript text must match its ordered segments".into());
    }
    Ok(())
}

fn validate_segments(
    transcript: &MediaTranscript,
    speaker_ids: &HashSet<&str>,
) -> Result<(), String> {
    if transcript.segments.len() > MAX_SEGMENTS {
        return Err("Transcript segments count is invalid".into());
    }
    let mut ids = HashSet::with_capacity(transcript.segments.len());
    let mut previous_start = None;
    for segment in &transcript.segments {
        require_sha256(&segment.id, "Transcript segment id")?;
        if !ids.insert(segment.id.as_str()) {
            return Err("Transcript segment ids must be unique".into());
        }
        require_revision(segment.revision, "Transcript segment revision")?;
        if segment.revision > transcript.revision {
            return Err("Transcript segment revision exceeds transcript revision".into());
        }
        require_safe_timestamp(segment.start_ms, "Transcript segment start")?;
        require_safe_timestamp(segment.end_ms, "Transcript segment end")?;
        if segment.end_ms <= segment.start_ms
            || segment.end_ms > transcript.duration_ms
            || previous_start.is_some_and(|previous| segment.start_ms < previous)
        {
            return Err("Transcript segments must be ordered and bounded".into());
        }
        previous_start = Some(segment.start_ms);
        let expected_start_sample = segment
            .start_ms
            .checked_mul(16)
            .ok_or("Transcript segment sample offset overflow")?;
        let expected_end_sample = segment
            .end_ms
            .checked_mul(16)
            .ok_or("Transcript segment sample offset overflow")?;
        require_safe_timestamp(segment.start_sample, "Transcript segment start sample")?;
        require_safe_timestamp(segment.end_sample, "Transcript segment end sample")?;
        if segment.start_sample != expected_start_sample
            || segment.end_sample != expected_end_sample
        {
            return Err("Transcript segment sample offsets must use the 16 kHz timeline".into());
        }
        require_bounded_text(&segment.text, 16_000, "Transcript segment text")?;
        if segment
            .speaker_id
            .as_deref()
            .is_some_and(|speaker| !speaker_ids.contains(speaker))
        {
            return Err("Transcript segment speakerId must reference a speaker track".into());
        }
        validate_units(
            &segment.text,
            segment.start_ms,
            segment.end_ms,
            &segment.units,
        )?;
    }
    Ok(())
}

fn validate_units(
    segment_text: &str,
    segment_start: u64,
    segment_end: u64,
    units: &[TranscriptUnit],
) -> Result<(), String> {
    if units.is_empty() || units.len() > MAX_UNITS {
        return Err("Transcript units count is invalid".into());
    }
    let characters = segment_text.chars().collect::<Vec<_>>();
    let mut ids = HashSet::with_capacity(units.len());
    let mut previous_end = 0_u32;
    for unit in units {
        require_sha256(&unit.id, "Transcript unit id")?;
        if !ids.insert(unit.id.as_str()) {
            return Err("Transcript unit ids must be unique within a segment".into());
        }
        require_raw_text(&unit.text, 16_000, "Transcript unit text")?;
        let (start, end) = validate_unit_range(unit, characters.len(), previous_end)?;
        if characters[start..end].iter().collect::<String>() != unit.text {
            return Err("Transcript unit text must match its character range".into());
        }
        previous_end = unit.end_character;
        validate_unit_kind(unit)?;
        if unit
            .score
            .is_some_and(|score| !score.is_finite() || !(0.0..=1.0).contains(&score))
        {
            return Err("Transcript unit score is invalid".into());
        }
        validate_unit_timing(unit, segment_start, segment_end)?;
    }
    Ok(())
}

fn validate_unit_range(
    unit: &TranscriptUnit,
    character_count: usize,
    previous_end: u32,
) -> Result<(usize, usize), String> {
    let start = usize::try_from(unit.start_character)
        .map_err(|_| "Transcript unit character range is invalid")?;
    let end = usize::try_from(unit.end_character)
        .map_err(|_| "Transcript unit character range is invalid")?;
    if end <= start || end > character_count || unit.start_character < previous_end {
        return Err("Transcript unit character ranges must be ordered and bounded".into());
    }
    Ok((start, end))
}

fn validate_unit_kind(unit: &TranscriptUnit) -> Result<(), String> {
    if !matches!(unit.unit_kind.as_str(), "word" | "character" | "segment") {
        return Err("Transcript unit kind is invalid".into());
    }
    Ok(())
}

fn validate_unit_timing(
    unit: &TranscriptUnit,
    segment_start: u64,
    segment_end: u64,
) -> Result<(), String> {
    match (unit.start_ms, unit.end_ms) {
        (Some(start_ms), Some(end_ms)) => {
            if end_ms <= start_ms
                || start_ms < segment_start
                || end_ms > segment_end
                || unit.unaligned_reason.is_some()
                || unit.unit_kind == "segment"
            {
                return Err("Aligned transcript unit timing is invalid".into());
            }
        }
        (None, None) => {
            let reason = unit
                .unaligned_reason
                .as_deref()
                .ok_or("Unaligned transcript unit requires a reason")?;
            require_bounded_text(reason, 120, "Transcript unit unalignedReason")?;
        }
        _ => return Err("Transcript unit timing must be fully aligned or fully absent".into()),
    }
    Ok(())
}

fn validate_cues(cues: &[SubtitleCue], duration: u64) -> Result<(), String> {
    if cues.len() > MAX_CUES {
        return Err("Transcript subtitleCues count is invalid".into());
    }
    let mut previous_start = None;
    for cue in cues {
        require_bounded_text(&cue.text, 16_000, "Subtitle cue text")?;
        require_safe_timestamp(cue.start_ms, "Subtitle cue start")?;
        require_safe_timestamp(cue.end_ms, "Subtitle cue end")?;
        if cue.end_ms <= cue.start_ms
            || cue.end_ms > duration
            || previous_start.is_some_and(|previous| cue.start_ms < previous)
        {
            return Err("Subtitle cues must be ordered and bounded".into());
        }
        previous_start = Some(cue.start_ms);
    }
    Ok(())
}

fn require_revision(value: u32, label: &str) -> Result<(), String> {
    if value == 0 {
        return Err(format!("{label} must be at least 1"));
    }
    Ok(())
}

fn require_sha256(value: &str, label: &str) -> Result<(), String> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!(
            "{label} must be a lowercase 64-character SHA-256 id"
        ));
    }
    Ok(())
}

fn require_safe_timestamp(value: u64, label: &str) -> Result<(), String> {
    if value > MAX_SAFE_INTEGER {
        return Err(format!("{label} exceeds the safe integer range"));
    }
    Ok(())
}

fn require_bounded_text(value: &str, maximum: usize, label: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.encode_utf16().count() > maximum {
        return Err(format!("{label} is invalid"));
    }
    Ok(())
}

fn require_raw_text(value: &str, maximum: usize, label: &str) -> Result<(), String> {
    if value.is_empty() || value.encode_utf16().count() > maximum {
        return Err(format!("{label} is invalid"));
    }
    Ok(())
}
