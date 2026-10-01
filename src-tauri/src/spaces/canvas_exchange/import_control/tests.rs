
use super::*;
#[test]
fn cancellation_reaches_resources_and_blocks_publication() {
    let id = uuid::Uuid::new_v4().to_string();
    let (token, _lease) = begin(&id).unwrap();
    let resource = token.child_token();
    assert!(check(&token).is_ok());
    cancel(&id).unwrap();
    assert!(resource.is_cancelled());
    assert!(check(&token)
        .unwrap_err()
        .starts_with("json-canvas-import-cancelled:"));
}
#[test]
fn completed_import_cannot_be_cancelled_and_duplicate_work_is_rejected() {
    let id = uuid::Uuid::new_v4().to_string();
    let (_, lease) = begin(&id).unwrap();
    assert!(begin(&id).is_err());
    drop(lease);
    assert!(jobs().get(&id).is_err());
    cancel(&id).unwrap();
}
