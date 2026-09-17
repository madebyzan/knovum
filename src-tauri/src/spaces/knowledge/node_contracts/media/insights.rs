use serde_json::{Map, Value};

use super::{nonnegative, transcript::TranscriptContract};
use crate::spaces::knowledge::node_contracts::scalars::required_string;
use crate::spaces::knowledge::node_contracts::validation::reject_unknown;

pub(super) fn validate(value: &Value, transcript: &TranscriptContract) -> Result<(), String> {
    let insights = value
        .as_object()
        .ok_or_else(|| "mediaKnowledge insights must be an object".to_string())?;
    reject_unknown(
        insights,
        [
            "summary",
            "chapters",
            "promptVersion",
            "generatedAt",
            "editedAt",
            "provider",
            "model",
        ],
        "Media insights",
    )?;
    required_string(insights, "summary", 16_000)?;
    if insights.get("promptVersion").and_then(Value::as_i64) != Some(1) {
        return Err("Media insights promptVersion must be 1".into());
    }
    nonnegative(insights, "generatedAt")?;
    if insights.contains_key("editedAt") {
        nonnegative(insights, "editedAt")?;
    }
    required_string(insights, "provider", 128)?;
    required_string(insights, "model", 256)?;
    validate_chapters(insights, transcript)
}

fn validate_chapters(
    insights: &Map<String, Value>,
    transcript: &TranscriptContract,
) -> Result<(), String> {
    let chapters = insights
        .get("chapters")
        .and_then(Value::as_array)
        .ok_or_else(|| "Media chapters must be an array".to_string())?;
    if chapters.is_empty() || chapters.len() > 2_000 {
        return Err("Media chapters count is invalid".into());
    }
    let mut previous = None;
    for (index, chapter) in chapters.iter().enumerate() {
        let start = validate_chapter(chapters, chapter, index, transcript, previous)?;
        previous = Some(start);
    }
    let first = chapters
        .first()
        .and_then(Value::as_object)
        .and_then(|value| value.get("startMs"))
        .and_then(Value::as_i64);
    if first != transcript.first_start {
        return Err("Chapters must start at the first transcript segment".into());
    }
    Ok(())
}

fn validate_chapter(
    chapters: &[Value],
    value: &Value,
    index: usize,
    transcript: &TranscriptContract,
    previous: Option<i64>,
) -> Result<i64, String> {
    let chapter = value
        .as_object()
        .ok_or_else(|| "Media chapter must be an object".to_string())?;
    reject_unknown(
        chapter,
        ["startMs", "endMs", "title", "summary"],
        "Media chapter",
    )?;
    let start = nonnegative(chapter, "startMs")?;
    let end = nonnegative(chapter, "endMs")?;
    required_string(chapter, "title", 180)?;
    required_string(chapter, "summary", 2_000)?;
    let expected_end = chapters
        .get(index + 1)
        .and_then(Value::as_object)
        .and_then(|next| next.get("startMs"))
        .and_then(Value::as_i64)
        .unwrap_or(transcript.duration);
    let invalid = end <= start
        || end > transcript.duration
        || !transcript.starts.contains(&start)
        || previous.is_some_and(|value| start <= value)
        || end != expected_end;
    if invalid {
        return Err(
            "Chapters must be ordered, contiguous, and anchored to transcript segments".into(),
        );
    }
    Ok(start)
}
