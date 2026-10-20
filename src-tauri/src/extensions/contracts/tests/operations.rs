use super::ExtensionChange;

#[test]
fn committed_change_keeps_value_and_collects_all_cleanup_failures() {
    let change = ExtensionChange::committed(
        "new-revision",
        [
            Err("credential-cleanup".into()),
            Ok(()),
            Err("content-cleanup".into()),
        ],
    );
    assert_eq!(change.value, "new-revision");
    assert_eq!(
        change.cleanup_errors,
        ["credential-cleanup", "content-cleanup"]
    );
    let wire = serde_json::to_value(change).unwrap();
    assert_eq!(wire["value"], "new-revision");
    assert_eq!(wire["cleanupErrors"].as_array().unwrap().len(), 2);
}
