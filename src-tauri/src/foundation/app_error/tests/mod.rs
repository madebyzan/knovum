use super::AppError;

#[test]
fn codes_are_explicit_and_messages_are_never_parsed() {
    for message in [
        "conversation-upload-size: too large",
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

#[test]
fn app_error_converts_into_string() {
    let error = AppError::invalid("invalid payload");
    let message: String = error.into();
    assert_eq!(message, "invalid payload");
}

#[test]
fn missing_configuration_is_told_by_its_code_not_its_message() {
    assert!(AppError::not_configured("No route").is_not_configured());
    assert!(!AppError::from("not_configured: written into a message").is_not_configured());
    assert!(!AppError::new("authentication_failed", "Rejected key").is_not_configured());
}
