use super::validate_endpoint;

#[test]
fn endpoint_requires_credential_free_http() {
    assert!(validate_endpoint("https://example.com/v1").is_ok());
    for value in [
        "ftp://example.com/v1",
        "https://user@example.com/v1",
        "https://user:secret@example.com/v1",
        "https://",
    ] {
        let error = validate_endpoint(value).unwrap_err();
        assert_eq!(error.message, "AI provider Base URL is invalid");
        assert!(!error.message.contains("secret"));
    }
}
