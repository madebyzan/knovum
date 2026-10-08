use super::*;
use crate::ai::{
    contracts::ExecutionCapability,
    tasks::completion::tests::{agent_target, ExecutorSpy},
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
    let image = std::path::PathBuf::from("/authorized/actual-edited-image.png");
    let interpretation = interpret(
        &spy,
        agent_target(ExecutionCapability::VisionUnderstanding),
        image,
        tokio_util::sync::CancellationToken::new(),
        &revision,
    )
    .await
    .unwrap();
    spy.assert_agent_only();
    {
        let inputs = spy.inputs.lock().unwrap();
        assert_eq!(inputs[0].max_tokens, Some(8192));
        for limit in [
            visual::MAX_DESCRIPTION,
            visual::MAX_DETECTED_TEXT,
            visual::MAX_TAGS,
            visual::MAX_TAG,
        ] {
            assert!(inputs[0].instruction.contains(&limit.to_string()));
        }
    }
    assert_eq!(
        spy.inputs.lock().unwrap()[0].image.as_deref(),
        Some(std::path::Path::new("/authorized/actual-edited-image.png"))
    );
    assert_eq!(
        serde_json::to_value(&interpretation).unwrap()["executionSource"],
        serde_json::json!({"type":"agent","runtimeKind":"codex"})
    );

    let pool = crate::spaces::assets::enrichment::commit::tests::fixture(&data).await;
    let published = crate::spaces::assets::enrichment::commit::tests::publish(
        &pool,
        &data,
        serde_json::Map::from_iter([("visualKnowledge".into(), interpretation.clone())]),
        &tokio_util::sync::CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(published["visualKnowledge"], interpretation);
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
            std::path::PathBuf::from("/authorized/image.png"),
            tokio_util::sync::CancellationToken::new(),
            "rendered-source-revision"
        )
        .await
        .is_err());
        spy.assert_agent_only();
    }
}

#[tokio::test]
async fn oversized_and_empty_visual_fields_are_rejected() {
    for (description, detected_text, tags, field) in [
        (" ".into(), vec![], vec![], "description"),
        ("x".repeat(8_001), vec![], vec![], "description"),
        (
            "Diagram".into(),
            vec!["x".repeat(16_001)],
            vec![],
            "detectedText",
        ),
        ("Diagram".into(), vec![], vec!["tag".into(); 33], "tags"),
        ("Diagram".into(), vec![], vec![" ".into()], "tags"),
        ("Diagram".into(), vec![], vec!["x".repeat(81)], "tags"),
    ] {
        let response = serde_json::json!({"description":description, "detectedText":detected_text, "tags":tags}).to_string();
        let spy = ExecutorSpy::returns(&response);
        let error = interpret(
            &spy,
            agent_target(ExecutionCapability::VisionUnderstanding),
            std::path::PathBuf::from("/authorized/image.png"),
            tokio_util::sync::CancellationToken::new(),
            "source",
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, "invalid_response");
        assert!(error.message.contains(&format!("data.{field}")));
    }
}

#[tokio::test]
async fn visual_limits_use_utf16_and_include_text_separators() {
    for (segments, valid) in [
        (vec!["x".repeat(7_999), "x".repeat(8_000)], true),
        (vec!["x".repeat(8_000), "x".repeat(8_000)], false),
    ] {
        let response = serde_json::json!({
            "description": "😀".repeat(4_000), "detectedText": segments,
            "tags": vec!["😀".repeat(40); 32],
        })
        .to_string();
        let spy = ExecutorSpy::returns(&response);
        let interpretation = interpret(
            &spy,
            agent_target(ExecutionCapability::VisionUnderstanding),
            std::path::PathBuf::from("/authorized/image.png"),
            tokio_util::sync::CancellationToken::new(),
            "source",
        )
        .await;
        assert_eq!(interpretation.is_ok(), valid);
    }
}

#[test]
fn web_downloaded_thumbnail_uses_space_asset_authority() {
    assert!(web_media_preview(
        &serde_json::json!({"origin": {"kind": "web"}})
    ));
    assert!(!web_media_preview(&serde_json::json!({"kind": "video"})));
}

#[tokio::test]
async fn inference_failure_keeps_its_code_and_message() {
    for code in [
        "authentication_failed",
        "agent_unavailable",
        "provider_request_failed",
        "cancelled",
    ] {
        let spy = ExecutorSpy::fails(crate::app_error::AppError::new(code, "Original failure"));
        let error = interpret(
            &spy,
            agent_target(ExecutionCapability::VisionUnderstanding),
            std::path::PathBuf::from("/authorized/image.png"),
            tokio_util::sync::CancellationToken::new(),
            "source",
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, code);
        assert_eq!(error.message, "Original failure");
        spy.assert_agent_only();
    }
}
