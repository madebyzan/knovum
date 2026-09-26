#[test]
fn prepared_presentation_metadata_accepts_resolved_action_fields() {
    let value = serde_json::json!({
        "id":"run", "spaceId":"space", "outputNodeId":"run-output",
        "inputs":[{"nodeId":"source", "role":"primary", "sourceRevision":"revision"}],
        "primary":{"nodeId":"source"},
        "document":{"nodeId":"source", "data":{"title":"Title", "text":"Content"}}
    });
    assert!(
        serde_json::from_value::<super::presentation_publication::PresentationPublication>(value)
            .is_ok()
    );
}
