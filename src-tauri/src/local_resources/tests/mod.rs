use super::configured_path::validate_configured_path;

#[test]
fn absent_override_allows_resolution_but_invalid_override_is_an_error() {
    assert!(validate_configured_path("KNOVUM_FFMPEG_PATH", None)
        .unwrap()
        .is_none());
    let directory = std::env::temp_dir().join(format!("knovum-config-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory).expect("directory");
    for path in [directory.clone(), directory.join("missing")] {
        let error = validate_configured_path("KNOVUM_FFMPEG_PATH", Some(path.into_os_string()))
            .expect_err("explicit invalid path must not use a bundled fallback");
        assert!(error.starts_with("local-resource-config-invalid:"));
        assert!(error.contains("KNOVUM_FFMPEG_PATH"));
    }
    let file = directory.join("ffmpeg");
    std::fs::write(&file, b"configured tool").expect("file");
    assert_eq!(
        validate_configured_path("KNOVUM_FFMPEG_PATH", Some(file.clone().into_os_string()))
            .expect("valid override"),
        Some(file)
    );
    std::fs::remove_dir_all(directory).expect("cleanup test directory");
}
