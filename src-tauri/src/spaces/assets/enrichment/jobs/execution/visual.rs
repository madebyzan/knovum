use serde::Deserialize;
use serde_json::Value;
use tauri::AppHandle;

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
) -> Result<(), String> {
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
        _ => return Err("Visual analysis requires an image or video node".to_owned()),
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
) -> Result<Value, String> {
    let completion = completion::execute(executor, target, input, cancellation)
        .await
        .map_err(|error| error.task_message())?;
    let analysis: VisualInterpretation =
        serde_json::from_str(completion.text.trim()).map_err(|error| {
            format!("ai-invalid-response: Visual analysis returned invalid JSON: {error}")
        })?;
    if analysis.description.trim().is_empty()
        || analysis.description.encode_utf16().count() > 8_000
        || analysis.tags.len() > 32
        || analysis
            .tags
            .iter()
            .any(|tag| tag.trim().is_empty() || tag.encode_utf16().count() > 80)
    {
        return Err(
            "ai-invalid-response: Visual interpretation exceeds its result contract".into(),
        );
    }
    let detected_text = analysis.detected_text.join("\n");
    if detected_text.encode_utf16().count() > 16_000 {
        return Err("ai-invalid-response: Visual detected text exceeds its result contract".into());
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

#[cfg(test)]
mod agent_task_tests {
    use super::*;
    use crate::ai::{
        contracts::ExecutionCapability,
        tasks::completion::tests::{agent_target, input, ExecutorSpy},
    };
    #[tokio::test]
    async fn agent_only_visual_uses_prepared_input_without_model_transport() {
        let data = serde_json::json!({"kind":"image","title":"Picture","fileName":"picture.png","assetPath":"/assets/picture.png","sourceRevision":"original-source"});
        let revision = super::super::super::revision::visual_revision(&data)
            .unwrap()
            .unwrap();
        let spy = ExecutorSpy::returns(
            r#"{"description":"Visible diagram","detectedText":["Evidence"],"tags":["diagram"]}"#,
        );
        let mut input = input();
        input.image = Some(std::path::PathBuf::from(
            "/authorized/actual-edited-image.png",
        ));
        let result = interpret(
            &spy,
            agent_target(ExecutionCapability::VisionUnderstanding),
            input,
            tokio_util::sync::CancellationToken::new(),
            &revision,
        )
        .await
        .unwrap();
        spy.assert_agent_only();
        assert_eq!(
            spy.inputs.lock().unwrap()[0].image.as_deref(),
            Some(std::path::Path::new("/authorized/actual-edited-image.png"))
        );
        assert_eq!(
            serde_json::to_value(&result).unwrap()["executionSource"],
            serde_json::json!({"type":"agent","runtimeKind":"codex"})
        );

        let pool = crate::spaces::assets::enrichment::commit::tests::fixture(&data).await;
        let published = crate::spaces::assets::enrichment::commit::tests::publish(
            &pool,
            &data,
            serde_json::Map::from_iter([("visualKnowledge".into(), result.clone())]),
            &tokio_util::sync::CancellationToken::new(),
        )
        .await
        .unwrap();
        assert_eq!(published["visualKnowledge"], result);
    }
    #[tokio::test]
    async fn invalid_visual_results_are_rejected() {
        for response in [
            "not JSON",
            "```json\n{}\n```",
            r#"{"description":"Invented but missing required fields"}"#,
        ] {
            let spy = ExecutorSpy::returns(response);
            assert!(interpret(
                &spy,
                agent_target(ExecutionCapability::VisionUnderstanding),
                input(),
                tokio_util::sync::CancellationToken::new(),
                "rendered-source-revision"
            )
            .await
            .is_err());
            spy.assert_agent_only();
        }
    }
}
