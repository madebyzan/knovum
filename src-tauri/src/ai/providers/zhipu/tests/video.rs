use super::parse_status;

#[test]
fn unknown_status_is_an_invalid_response() {
    let error = parse_status("QUEUED", &serde_json::json!({})).unwrap_err();
    assert_eq!(error.code, "invalid_response");
}
