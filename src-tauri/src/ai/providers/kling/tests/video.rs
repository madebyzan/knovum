use super::parse_status;

#[test]
fn recognizes_processing_and_rejects_unknown_status() {
    assert!(parse_status("processing", &serde_json::json!({})).is_ok());
    let error = parse_status("paused", &serde_json::json!({})).unwrap_err();
    assert_eq!(error.code, "invalid_response");
}
