use serde::Deserialize;
use serde_json::Value;
use tauri::AppHandle;

use crate::{
    ai::{
        capabilities::vision,
        models::{ExecutionTarget, ImageAnalysisRequest},
    },
    spaces::{assets::authorize_spaces_asset, previews::authorize_generated_preview},
};

use super::{node::required_string, RunEnrichmentJobRequest};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VisualAnalysisResult {
    description: String,
    detected_text: Option<Vec<String>>,
    tags: Vec<String>,
}

pub(super) async fn analyze(
    app: &AppHandle,
    request: &RunEnrichmentJobRequest,
    data: &mut Value,
    target: ExecutionTarget,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<(), String> {
    let source = match request.job_type {
        super::EnrichmentJobType::AnalyzeImage => authorize_spaces_asset(
            app,
            std::path::Path::new(required_string(data, "assetPath")?),
        )?,
        super::EnrichmentJobType::AnalyzeVideoCover => {
            let path = std::path::Path::new(required_string(data, "previewPath")?);
            if web_media_preview(data) {
                authorize_spaces_asset(app, path)?
            } else {
                authorize_generated_preview(app, path)?
            }
        }
        _ => return Err("Visual analysis received a non-visual job".to_owned()),
    };
    let completion = vision::analyze_image(app, ImageAnalysisRequest {
            target,
            image_path: source.to_string_lossy().into_owned(),
            system_prompt: "You analyze images for a local-first knowledge space. Return strict JSON only.".to_owned(),
            instruction: "Analyze this image for semantic retrieval. Return JSON with fields description, detectedText, and tags. detectedText must be an array of visible text segments in reading order.".to_owned(),
            max_tokens: None,
        }, cancellation)
    .await
    .map_err(|error| error.to_string())?;
    let analysis: VisualAnalysisResult = serde_json::from_str(completion.text.trim())
        .map_err(|error| format!("Visual analysis returned invalid JSON: {error}"))?;
    let detected_text = analysis.detected_text.unwrap_or_default().join("\n");
    let object = data
        .as_object_mut()
        .ok_or_else(|| "Space node data must be an object".to_owned())?;
    let mut knowledge = serde_json::json!({
        "sourceRevision": request.source_revision,
        "description": analysis.description,
        "tags": analysis.tags,
        "generatedAt": crate::clock::unix_time_millis()?,
        "provider": completion.provider,
        "model": completion.model,
    });
    if !detected_text.is_empty() {
        knowledge["detectedText"] = Value::from(detected_text);
    }
    object.insert("visualKnowledge".to_owned(), knowledge);
    Ok(())
}

fn web_media_preview(data: &Value) -> bool {
    data.get("origin")
        .and_then(|origin| origin.get("kind"))
        .and_then(Value::as_str)
        == Some("web")
}

#[cfg(test)]
mod preview_authority_tests {
    use super::web_media_preview;

    #[test]
    fn web_downloaded_thumbnail_uses_space_asset_authority() {
        assert!(web_media_preview(
            &serde_json::json!({"origin": {"kind": "web"}})
        ));
        assert!(!web_media_preview(&serde_json::json!({"kind": "video"})));
    }
}
