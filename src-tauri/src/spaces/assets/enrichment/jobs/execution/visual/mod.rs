use serde::Deserialize;
use serde_json::Value;
use tauri::AppHandle;

use crate::app_error::AppError;
use crate::{
    ai::tasks::{
        completion::{self, TaskInput},
        TaskExecutionTarget,
    },
    spaces::{assets::authorize_spaces_asset, previews::authorize_generated_preview},
};

use super::{node::required_string, RunEnrichmentJobRequest};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VisualInterpretation {
    description: String,
    detected_text: Vec<String>,
    tags: Vec<String>,
}

pub(super) async fn analyze(
    app: &AppHandle,
    request: &RunEnrichmentJobRequest,
    data: &mut Value,
    target: TaskExecutionTarget,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<(), AppError> {
    let source = match data.get("kind").and_then(Value::as_str) {
        Some("image") => crate::spaces::assets::image::rendered_preview(app, data).await?,
        Some("video") => {
            let path = std::path::Path::new(required_string(data, "previewPath")?);
            if web_media_preview(data) {
                authorize_spaces_asset(app, path)?
            } else {
                authorize_generated_preview(app, path)?
            }
        }
        _ => {
            return Err(AppError::invalid(
                "Visual analysis requires an image or video node",
            ))
        }
    };
    let completion = interpret(&completion::ProductExecutor(app), target, TaskInput {
            purpose: crate::ai::usage::UsagePurpose::VisualAnalysis,
            text: String::new(),
            image: Some(source),
            system_prompt: "You analyze images for a local-first knowledge space. Return strict JSON only.".to_owned(),
            instruction: "Analyze this image for semantic retrieval. Return JSON with fields description, detectedText, and tags. detectedText must be an array of visible text segments in reading order.".to_owned(),
            max_tokens: None,
        }, cancellation.clone(), &request.source_revision)
    .await
    ?;
    let object = data
        .as_object_mut()
        .ok_or("Space node data must be an object")?;
    object.insert("visualKnowledge".into(), completion);
    Ok(())
}

async fn interpret(
    executor: &impl completion::TaskExecutor,
    target: TaskExecutionTarget,
    input: TaskInput,
    cancellation: tokio_util::sync::CancellationToken,
    revision: &str,
) -> Result<Value, AppError> {
    let completion = completion::execute(executor, target, input, cancellation).await?;
    let analysis: VisualInterpretation =
        serde_json::from_str(completion.text.trim()).map_err(|error| {
            AppError::new(
                "invalid_response",
                format!("Visual analysis returned invalid JSON: {error}"),
            )
        })?;
    if analysis.description.trim().is_empty()
        || analysis.description.encode_utf16().count() > 8_000
        || analysis.tags.len() > 32
        || analysis
            .tags
            .iter()
            .any(|tag| tag.trim().is_empty() || tag.encode_utf16().count() > 80)
    {
        return Err(AppError::new(
            "invalid_response",
            "Visual interpretation exceeds its result contract",
        ));
    }
    let detected_text = analysis.detected_text.join("\n");
    if detected_text.encode_utf16().count() > 16_000 {
        return Err(AppError::new(
            "invalid_response",
            "Visual detected text exceeds its result contract",
        ));
    }
    let mut knowledge = serde_json::json!({
        "sourceRevision": revision,
        "description": analysis.description,
        "tags": analysis.tags,
        "generatedAt": crate::clock::unix_time_millis()?,
        "executionSource": completion.execution_source,
    });
    if !detected_text.is_empty() {
        knowledge["detectedText"] = Value::from(detected_text);
    }
    Ok(knowledge)
}

fn web_media_preview(data: &Value) -> bool {
    data.get("origin")
        .and_then(|origin| origin.get("kind"))
        .and_then(Value::as_str)
        == Some("web")
}

#[cfg(test)]
mod tests;
