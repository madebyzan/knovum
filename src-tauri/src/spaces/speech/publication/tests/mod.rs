use super::cleanup_failed_output;

#[test]
fn cleanup_preserves_the_original_failure_and_exposes_failed_deletion() {
    let path = std::env::temp_dir().join(format!("knovum-cleanup-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&path).unwrap();
    let failure = cleanup_failed_output(&path, "digest differs".into());
    assert!(failure.starts_with("digest differs; failed to remove "));
    std::fs::remove_dir(&path).unwrap();
    assert_eq!(
        cleanup_failed_output(&path, "digest differs".into()),
        "digest differs"
    );
}
