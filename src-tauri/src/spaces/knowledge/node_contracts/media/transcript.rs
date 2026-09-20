use std::collections::HashSet;

use serde_json::{Map, Value};

use super::nonnegative;
use crate::spaces::knowledge::node_contracts::scalars::{
    required_string, required_string_allow_empty,
};
use crate::spaces::knowledge::node_contracts::validation::reject_unknown;

const MAX_TEXT: usize = 4 * 1024 * 1024;

pub(super) struct TranscriptContract {
    pub(super) duration: i64,
    pub(super) starts: HashSet<i64>,
    pub(super) first_start: Option<i64>,
}

pub(super) fn validate(object: &Map<String, Value>) -> Result<TranscriptContract, String> {
    let transcript = object
        .get("transcript")
        .and_then(Value::as_object)
        .ok_or_else(|| "mediaKnowledge transcript is required".to_string())?;
    reject_unknown(
        transcript,
        ["language", "durationMs", "text", "segments", "subtitleCues"],
        "media transcript",
    )?;
    required_string(transcript, "language", 64)?;
    let duration = nonnegative(transcript, "durationMs")?;
    required_string_allow_empty(transcript, "text", MAX_TEXT)?;
    let segments = transcript
        .get("segments")
        .and_then(Value::as_array)
        .ok_or_else(|| "Transcript segments must be an array".to_string())?;
    if segments.len() > 100_000 {
        return Err("Transcript segments count is invalid".into());
    }
    let mut previous = 0_i64;
    let mut starts = HashSet::new();
    for (index, segment) in segments.iter().enumerate() {
        let start = validate_segment(segment, duration, previous, index > 0)?;
        previous = start;
        starts.insert(start);
    }
    let first_start = segments
        .first()
        .and_then(Value::as_object)
        .and_then(|value| value.get("startMs"))
        .and_then(Value::as_i64);
    validate_subtitle_cues(transcript, duration)?;
    Ok(TranscriptContract {
        duration,
        starts,
        first_start,
    })
}

fn validate_subtitle_cues(transcript: &Map<String, Value>, duration: i64) -> Result<(), String> {
    let cues = transcript
        .get("subtitleCues")
        .and_then(Value::as_array)
        .ok_or_else(|| "Transcript subtitleCues must be an array".to_string())?;
    if cues.len() > 100_000 {
        return Err("Transcript subtitleCues count is invalid".into());
    }
    let mut previous = 0;
    for (index, value) in cues.iter().enumerate() {
        let cue = value
            .as_object()
            .ok_or_else(|| "Subtitle cue must be an object".to_string())?;
        reject_unknown(cue, ["startMs", "endMs", "text"], "Subtitle cue")?;
        let start = nonnegative(cue, "startMs")?;
        let end = nonnegative(cue, "endMs")?;
        required_string(cue, "text", 16_000)?;
        if end <= start || end > duration || (index > 0 && start < previous) {
            return Err("Subtitle cues must be ordered and bounded".into());
        }
        previous = start;
    }
    Ok(())
}

fn validate_segment(
    value: &Value,
    duration: i64,
    previous: i64,
    has_previous: bool,
) -> Result<i64, String> {
    let segment = value
        .as_object()
        .ok_or_else(|| "Transcript segment must be an object".to_string())?;
    reject_unknown(segment, ["startMs", "endMs", "text"], "Transcript segment")?;
    let start = nonnegative(segment, "startMs")?;
    let end = nonnegative(segment, "endMs")?;
    required_string(segment, "text", 16_000)?;
    if end < start || end > duration || (has_previous && start < previous) {
        return Err("Transcript segments must be ordered and bounded".into());
    }
    Ok(start)
}
