
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
