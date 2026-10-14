#[test]
fn committed_import_receipt_survives_marker_cleanup_failure() {
    let marker = std::env::temp_dir().join(format!(
        "knovum-missing-package-marker-{}",
        uuid::Uuid::new_v4()
    ));

    assert_eq!(
        super::finalize_committed_import("overview", &marker, "space"),
        "overview"
    );
}
