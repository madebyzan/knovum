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

#[test]
fn explicitly_coded_errors_preserve_their_message() {
    let error = CommandError::new(
        "media_insights_cancelled",
        "Media insights generation cancelled",
    );
    assert_eq!(error.code, "media_insights_cancelled");
    assert_eq!(error.message, "Media insights generation cancelled");
}

#[test]
fn space_edit_conflicts_keep_their_code_and_message() {
    let message = "space-edit-revision-conflict: Cross-Space edit revision conflict";
    let error = CommandError::from_space_edit(message.to_owned());
    assert_eq!(error.code, "space_edit_revision_conflict");
    assert_eq!(error.message, message);
    assert_eq!(
        CommandError::from_space_edit("Space edit is unavailable".into()).code,
        "operation_failed"
    );
}
