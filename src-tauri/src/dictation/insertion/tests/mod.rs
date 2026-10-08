use super::{InsertionOutcome, PasteError};

#[test]
fn paste_error_classification_does_not_depend_on_message_text() {
    let permission = PasteError::permission_denied("localized permission message");
    assert!(matches!(
        permission.outcome(),
        InsertionOutcome::PermissionDenied
    ));
    assert_eq!(permission.message(), "localized permission message");

    let failed = PasteError::failed("permission_denied: ordinary failure");
    assert!(matches!(failed.outcome(), InsertionOutcome::Failed));
    assert_eq!(failed.message(), "permission_denied: ordinary failure");
}
