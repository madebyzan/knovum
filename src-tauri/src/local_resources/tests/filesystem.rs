use super::*;

#[test]
fn removal_rejects_a_foreign_parent_and_preserves_its_files() {
    let parent =
        std::env::temp_dir().join(format!("knovum-model-removal-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&parent).unwrap();
    let target = parent.join("snapshot");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("model.onnx"), b"owned").unwrap();
    assert!(remove_owned_directory(&target, &target).is_err());
    assert_eq!(fs::read(target.join("model.onnx")).unwrap(), b"owned");
    remove_owned_directory(&target, &parent).unwrap();
    assert!(!target.exists());
    remove_owned_directory(&target, &parent).unwrap();
    fs::remove_dir(parent).unwrap();
}

#[cfg(unix)]
#[test]
fn removal_unlinks_a_snapshot_without_following_its_external_target() {
    let parent = std::env::temp_dir().join(format!("knovum-model-link-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&parent).unwrap();
    let external = parent.join("external");
    fs::create_dir(&external).unwrap();
    fs::write(external.join("model.onnx"), b"preserve").unwrap();
    let link = parent.join("snapshot");
    std::os::unix::fs::symlink(&external, &link).unwrap();
    assert!(verify_regular_directory(&link).is_err());
    remove_owned_directory(&link, &parent).unwrap();
    assert_eq!(fs::read(external.join("model.onnx")).unwrap(), b"preserve");
    fs::remove_dir_all(parent).unwrap();
}
