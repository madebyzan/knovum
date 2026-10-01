use super::AppError;

#[test]
fn codes_are_explicit_and_messages_are_never_parsed() {
    for message in [
        "assistant-upload-size: too large",
        "authorization-grant-expired",
        "failed",
    ] {
        let error = AppError::from(message);
        assert_eq!(error.code, "operation_failed");
        assert_eq!(error.message, message);
    }
}

#[test]
fn all_product_failures_use_the_same_wire_contract() {
    for code in [
        "not_configured",
        "authentication_failed",
        "space_edit_revision_conflict",
        "cancelled",
    ] {
        let error = AppError::new(code, "Failure detail");
        assert_eq!(
            serde_json::to_value(error).unwrap(),
            serde_json::json!({"code":code,"message":"Failure detail"})
        );
    }
}
