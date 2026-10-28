use super::*;

#[test]
fn projection_is_optional_strict_and_rejects_symlinks() {
    let root = std::env::temp_dir().join(format!("agent-projection-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    assert!(read(&root).unwrap().producers.is_empty());

    fs::write(root.join(MANIFEST_NAME), r#"{"version":1,"producers":{}}"#).unwrap();
    assert!(read(&root)
        .err()
        .unwrap()
        .to_string()
        .contains("accepts v2"));

    fs::write(
        root.join(MANIFEST_NAME),
        r#"{"version":2,"producers":{},"extra":true}"#,
    )
    .unwrap();
    assert!(read(&root).is_err());

    #[cfg(unix)]
    {
        fs::remove_file(root.join(MANIFEST_NAME)).unwrap();
        let outside = root.with_extension("json");
        fs::write(&outside, r#"{"version":2,"producers":{}}"#).unwrap();
        std::os::unix::fs::symlink(&outside, root.join(MANIFEST_NAME)).unwrap();
        assert!(read(&root).is_err());
        fs::remove_file(outside).unwrap();
    }
    fs::remove_dir_all(root).unwrap();
}
