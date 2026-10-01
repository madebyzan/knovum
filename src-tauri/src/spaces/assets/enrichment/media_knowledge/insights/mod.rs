use crate::app_error::AppError;
use crate::{
    ai::{
        self,
        tasks::completion::{execute, ProductExecutor, TaskExecutor, TaskInput},
    },
    spaces::assets::transcription::{self, MediaTranscriptionProgress},
};
use futures::{stream, StreamExt};
use serde_json::{json, Value};
use tauri::{ipc::Channel, AppHandle};
use tokio_util::sync::CancellationToken;

pub(super) async fn generate(
    app: &AppHandle,
    selection: Option<ai::tasks::TaskTargetSelection>,
    knowledge: &Value,
    progress: &Channel<MediaTranscriptionProgress>,
    cancellation: &CancellationToken,
) -> Result<Value, AppError> {
    let selection = match selection {
        Some(selected) => selected,
        None => ai::settings::default_selection(app, ai::contracts::RoutingPurpose::Knowledge)?,
    };
    let target = ai::settings::resolve_task(
        app,
        selection,
        ai::contracts::ExecutionCapability::TextGeneration,
    )?;
    generate_with(
        &ProductExecutor(app),
        target,
        knowledge,
        progress,
        cancellation,
    )
    .await
}

pub(super) async fn generate_with(
    executor: &impl TaskExecutor,
    target: ai::tasks::TaskExecutionTarget,
    knowledge: &Value,
    progress: &Channel<MediaTranscriptionProgress>,
    cancellation: &CancellationToken,
) -> Result<Value, AppError> {
    let transcript = &knowledge["transcript"];
    let chunks = transcript_chunks(transcript)?;
    transcription::send_progress(progress, "organizing", 10);
    let chunk_cancellation = cancellation.child_token();
    let completions = stream::iter(chunks.into_iter().map(|text| {
        let target = target.clone();
        let cancellation = chunk_cancellation.clone();
        async move {
            let result = execute(
                executor,
                target,
                TaskInput {
                    purpose: ai::usage::UsagePurpose::Knowledge,
                    system_prompt: String::new(),
                    instruction: instruction(false),
                    text,
                    max_tokens: None,
                    image: None,
                },
                cancellation.clone(),
            )
            .await;
            if result.is_err() {
                cancellation.cancel();
            }
            result
        }
    }))
    .buffered(3)
    // Drain started sessions so cancellation is acknowledged before workspace cleanup.
    .collect::<Vec<_>>()
    .await;
    if let Some(error) = completions
        .iter()
        .filter_map(|result| result.as_ref().err())
        .find(|error| error.code != "cancelled")
    {
        return Err(error.clone());
    }
    let completions = completions.into_iter().collect::<Result<Vec<_>, _>>()?;
    let mut candidates = Vec::with_capacity(completions.len());
    for completion in &completions {
        candidates.push(parse_candidate(&completion.text).map_err(invalid_result)?);
    }
    let first = completions.first().ok_or("Transcript has no content")?;
    let mut execution_source = first.execution_source.clone();
    let candidate = if candidates.len() == 1 {
        candidates.remove(0)
    } else {
        transcription::send_progress(progress, "organizing", 70);
        let completion = execute(
            executor,
            target,
            TaskInput {
                purpose: ai::usage::UsagePurpose::Knowledge,
                system_prompt: String::new(),
                instruction: instruction(true),
                text: candidate_text(&candidates)?,
                max_tokens: None,
                image: None,
            },
            cancellation.clone(),
        )
        .await?;
        execution_source = completion.execution_source;
        parse_candidate(&completion.text).map_err(invalid_result)?
    };
    let mut next = knowledge.clone();
    next["insights"] = json!({"summary":candidate["summary"],"chapters":chapters(&candidate["chapters"], transcript).map_err(invalid_result)?,"promptVersion":1,"generatedAt":crate::clock::unix_time_millis()?,"executionSource":execution_source});
    transcription::send_progress(progress, "organizing", 95);
    Ok(next)
}

fn transcript_chunks(transcript: &Value) -> Result<Vec<String>, String> {
    let segments = transcript["segments"]
        .as_array()
        .ok_or("Transcript segments are unavailable")?;
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut length = 0;
    for segment in segments {
        let start = segment["startMs"]
            .as_u64()
            .ok_or("Invalid transcript timestamp")?;
        let text = segment["text"].as_str().ok_or("Invalid transcript text")?;
        let line = format!("[{start}ms] {text}");
        let line_length = line.encode_utf16().count();
        if !current.is_empty() && length + line_length + 1 > 14_000 {
            chunks.push(std::mem::take(&mut current));
            length = 0;
        }
        if !current.is_empty() {
            current.push('\n');
            length += 1;
        }
        current.push_str(&line);
        length += line_length;
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    if chunks.is_empty() {
        return Err("Transcript has no content".into());
    }
    Ok(chunks)
}

fn instruction(final_pass: bool) -> String {
    let source = if final_pass {
        "chapter candidates created from the complete transcript"
    } else {
        "timestamped transcript"
    };
    format!("Structure the {source} into a searchable media knowledge record. Return JSON only, with exactly this shape: {{\"summary\":\"...\",\"chapters\":[{{\"startMs\":0,\"title\":\"...\",\"summary\":\"...\"}}]}} summary must capture the whole supplied material concisely. chapters must follow content order, use timestamps that exist in the supplied material, and describe topic changes rather than arbitrary time intervals. The first chapter must start at the first timestamp in the supplied material. Use the language of the supplied material. Do not add markdown or commentary outside the JSON object.")
}

fn invalid_result(message: String) -> AppError {
    AppError::new("invalid_response", message)
}

fn parse_candidate(text: &str) -> Result<Value, String> {
    let value: Value =
        serde_json::from_str(text).map_err(|e| format!("Invalid media insights JSON: {e}"))?;
    let fields = value
        .as_object()
        .ok_or("Media insights must be an object")?;
    if fields.len() != 2 || !fields.contains_key("summary") || !fields.contains_key("chapters") {
        return Err("Invalid media insights fields".into());
    }
    bounded_text(&value["summary"], 16_000)?;
    let chapters = value["chapters"]
        .as_array()
        .filter(|v| !v.is_empty() && v.len() <= 2000)
        .ok_or("Invalid media insights chapters")?;
    for chapter in chapters {
        let fields = chapter
            .as_object()
            .ok_or("Media chapter must be an object")?;
        if fields.len() != 3
            || fields
                .keys()
                .any(|key| !["startMs", "title", "summary"].contains(&key.as_str()))
        {
            return Err("Invalid media chapter fields".into());
        }
        bounded_text(&chapter["title"], 180)?;
        bounded_text(&chapter["summary"], 2000)?;
        chapter["startMs"]
            .as_u64()
            .ok_or("Invalid chapter timestamp")?;
    }
    Ok(value)
}
fn bounded_text(value: &Value, max: usize) -> Result<(), String> {
    let text = value.as_str().ok_or("Media insight text is required")?;
    if text.trim().is_empty() || text.encode_utf16().count() > max {
        return Err("Media insight text exceeds its contract".into());
    }
    Ok(())
}
fn candidate_text(candidates: &[Value]) -> Result<String, String> {
    candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            let summary = candidate["summary"]
                .as_str()
                .ok_or("Missing candidate summary")?;
            let chapters = candidate["chapters"]
                .as_array()
                .ok_or("Missing candidate chapters")?;
            let lines = chapters
                .iter()
                .map(|chapter| {
                    let start = chapter["startMs"]
                        .as_u64()
                        .ok_or("Missing chapter timestamp")?;
                    let title = chapter["title"].as_str().ok_or("Missing chapter title")?;
                    let summary = chapter["summary"]
                        .as_str()
                        .ok_or("Missing chapter summary")?;
                    Ok(format!("[{start}ms] {title}: {summary}"))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(format!(
                "Part {}\nSummary: {summary}\n{}",
                index + 1,
                lines.join("\n")
            ))
        })
        .collect::<Result<Vec<_>, String>>()
        .map(|parts| parts.join("\n\n"))
}
pub(crate) fn chapters(candidates: &Value, transcript: &Value) -> Result<Vec<Value>, String> {
    let mut chapters = candidates
        .as_array()
        .ok_or("Invalid chapter candidates")?
        .clone();
    chapters.sort_by_key(|v| v["startMs"].as_u64());
    let segments = transcript["segments"]
        .as_array()
        .ok_or("Invalid transcript")?;
    let starts: std::collections::HashSet<_> = segments
        .iter()
        .filter_map(|v| v["startMs"].as_u64())
        .collect();
    if chapters.is_empty()
        || chapters[0]["startMs"] != segments.first().ok_or("Empty transcript")?["startMs"]
    {
        return Err("Media chapters must start with the transcript".into());
    }
    let duration = transcript["durationMs"]
        .as_u64()
        .ok_or("Invalid transcript duration")?;
    for i in 0..chapters.len() {
        let start = chapters[i]["startMs"]
            .as_u64()
            .ok_or("Invalid chapter timestamp")?;
        let end = if i + 1 < chapters.len() {
            chapters[i + 1]["startMs"]
                .as_u64()
                .ok_or("Invalid chapter timestamp")?
        } else {
            duration
        };
        if !starts.contains(&start) || end <= start || end > duration {
            return Err("Media chapters must use unique, ordered transcript timestamps".into());
        }
        chapters[i]["endMs"] = json!(end);
    }
    Ok(chapters)
}

#[cfg(test)]
mod tests;
