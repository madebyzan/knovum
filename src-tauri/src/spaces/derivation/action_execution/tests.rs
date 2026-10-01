use super::*;
use serde_json::json;
fn prepared() -> Value {
    json!({"outputNodeId":"output", "inputs":[{"nodeId":"note","role":"primary","sourceRevision":"knowledge:1"}], "target":{"type":"agent","runtimeKind":"codex"}})
}
#[test]
fn text_action_parameters_project_to_strict_engine_requests() {
    for (action, parameters) in [
        ("draft-document", json!({"title":"Doc","purpose":"Explain"})),
        ("draft-script", json!({"title":"Script","prompt":"Explain"})),
        ("summarize-meeting", json!({"summaryPurpose":"Explain"})),
        (
            "draft-social-post",
            json!({"platform":"Web","audience":"Readers","goal":"Explain","tone":"Clear"}),
        ),
    ] {
        let value = text_request(
            action,
            request_value("run", "space", &parameters, &prepared()).unwrap(),
        )
        .unwrap();
        decode::<grounded_document::GenerateGroundedDocumentRequest>(value).unwrap();
    }
}
#[test]
fn image_and_video_actions_project_to_strict_engine_requests() {
    let value = image_request(
        request_value(
            "run",
            "space",
            &json!({"title":"Image", "intent":"Explain", "style":"Flat", "aspectRatio":"1:1"}),
            &prepared(),
        )
        .unwrap(),
    )
    .unwrap();
    decode::<visualization::GenerateVisualizationRequest>(value).unwrap();
    let value = request_value(
        "run",
        "space",
        &json!({"title":"Video","prompt":"Explain","aspectRatio":"16:9","durationSeconds":5}),
        &prepared(),
    )
    .unwrap();
    decode::<video_generation::GenerateVideoRequest>(value).unwrap();
}
