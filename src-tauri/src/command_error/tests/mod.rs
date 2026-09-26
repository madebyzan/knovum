use super::CommandError;

#[test]
fn stable_domain_codes_survive_the_ipc_boundary_without_parsing_english_messages() {
    let error = CommandError::from("assistant-upload-size: attachment exceeds the limit");
    assert_eq!(
        serde_json::to_value(error).expect("error response"),
        serde_json::json!({
            "code": "assistant_upload_size", "message": "assistant-upload-size: attachment exceeds the limit"
        })
    );
    assert_eq!(
        CommandError::from("authorization-grant-expired").code,
        "authorization_grant_expired"
    );
    assert_eq!(
        CommandError::from("Failed to read file: denied").code,
        "operation_failed"
    );
}
