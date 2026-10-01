use super::*;
use crate::ai::{
    contracts::ExecutionCapability,
    tasks::completion::tests::{agent_target, input, ExecutorSpy},
};
#[tokio::test]
async fn agent_only_visual_uses_prepared_input_without_model_transport() {
    let spy =
        ExecutorSpy::returns(r#"{"description":"Visible diagram","detectedText":["Evidence"]}"#);
    let mut input = input();
    input.image = Some(std::path::PathBuf::from(
        "/authorized/actual-edited-image.png",
    ));
    let result = analyze_input(
        &spy,
        agent_target(ExecutionCapability::VisionUnderstanding),
        input,
        CancellationToken::new(),
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

    let coverage = serde_json::json!({"schemaVersion":1,"parser":"office-oxide","sourceRevision":"source","pageCount":null,"pagesRequiringRecognition":[],"pagesWithExtractedText":[],"pagesWithoutExtractableContent":[],"resources":[{"id":"a".repeat(64),"path":"/resources/region.png","mediaType":"image/png","width":100,"height":100,"byteSize":1000}],"occurrences":[{"id":"region","location":{"kind":"word","section":0,"element":1,"headingPath":[]},"context":"Body","altText":null,"resourceId":"a".repeat(64),"pdfInput":null,"state":"pending","error":null,"analysis":null}],"warnings":[]});
    let data = serde_json::json!({"kind":"file","title":"Document","fileName":"document.docx","assetPath":"/assets/document.docx","sourceRevision":"source","documentCoverage":coverage});
    let pool = crate::spaces::assets::enrichment::commit::tests::fixture(&data).await;
    let mut next = data["documentCoverage"].clone();
    next["occurrences"][0]["state"] = serde_json::json!("completed");
    next["occurrences"][0]["analysis"] = serde_json::to_value(&result).unwrap();
    let published = crate::spaces::assets::enrichment::commit::tests::publish(
        &pool,
        &data,
        serde_json::Map::from_iter([("documentCoverage".into(), next.clone())]),
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(published["documentCoverage"], next);
    assert_eq!(
        published["documentCoverage"]["occurrences"][0]["location"],
        data["documentCoverage"]["occurrences"][0]["location"]
    );
}
#[tokio::test]
async fn invalid_visual_results_are_rejected() {
    for response in [
        "not JSON",
        "```json\n{}\n```",
        r#"{"description":"Invented but missing required fields"}"#,
    ] {
        let spy = ExecutorSpy::returns(response);
        assert!(analyze_input(
            &spy,
            agent_target(ExecutionCapability::VisionUnderstanding),
            input(),
            CancellationToken::new()
        )
        .await
        .is_err());
        spy.assert_agent_only();
    }
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
        let error = analyze_input(
            &spy,
            agent_target(ExecutionCapability::VisionUnderstanding),
            input(),
            CancellationToken::new(),
        )
        .await
        .err()
        .expect("inference must fail");
        assert_eq!(error.code, code);
        assert_eq!(error.message, "Original failure");
        spy.assert_agent_only();
    }
}
