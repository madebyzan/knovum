use super::*;

#[test]
fn attempt_artifact_path_accepts_one_ascii_file_beneath_matching_attempt() {
    let root = std::env::temp_dir().join(format!("knovum-artifact-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(root.join("attempt-request")).unwrap();
    assert_eq!(
        attempt_artifact_path(&root, "request", "attempt-request/audio_1.wav").unwrap(),
        root.join("attempt-request/audio_1.wav")
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn attempt_artifact_path_rejects_mismatched_nested_and_non_ascii_names() {
    let root = std::path::Path::new("/runtime");
    for name in [
        "attempt-other/audio.wav",
        "attempt-request/nested/audio.wav",
        "attempt-request/../audio.wav",
        "attempt-request/音频.wav",
    ] {
        assert!(
            attempt_artifact_path(root, "request", name).is_err(),
            "{name}"
        );
    }
}
