use serde_json::json;

use super::build;

#[test]
fn an_image_only_request_needs_no_knowledge_section() {
    let prompt = build("Make it snow", &[], 1).unwrap();
    assert!(prompt.contains("ATTACHED IMAGES: 1"));
    assert!(prompt.contains("Make it snow"));
    assert!(
        !prompt.contains("KNOWLEDGE"),
        "a selection without text must not claim knowledge: {prompt}"
    );
}

#[test]
fn a_text_only_request_quotes_its_sources() {
    let prompt = build(
        "Explain the flow",
        &[json!({"nodeTitle": "Spec", "text": "Step one."})],
        0,
    )
    .unwrap();
    assert!(prompt.contains("SOURCE 1 — Spec"));
    assert!(prompt.contains("Step one."));
    assert!(
        !prompt.contains("ATTACHED IMAGES"),
        "no image was attached: {prompt}"
    );
}

#[test]
fn a_source_without_text_is_rejected() {
    assert!(build("Intent", &[json!({"nodeTitle": "Spec"})], 0).is_err());
}
