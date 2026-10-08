use super::*;

#[test]
fn extracted_resources_are_owned_by_the_source_asset() {
    let assets = Path::new("assets");
    let first = resource_path(assets, Path::new("assets/first.docx"), "digest");
    let second = resource_path(assets, Path::new("assets/second.docx"), "digest");
    assert_ne!(first, second);
    assert_eq!(
        first,
        resource_path(assets, Path::new("assets/first.docx"), "digest")
    );
    assert_ne!(
        first,
        resource_path(assets, Path::new("assets/first.docx"), "changed")
    );
}

#[test]
fn ownership_is_claimed_before_resource_bytes_are_created() {
    let directory =
        std::env::temp_dir().join(format!("document-ownership-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("resource.png");
    let mut claimed = Vec::new();
    let resource = store_at(
        &path,
        hex::encode(Sha256::digest(b"png")),
        b"png",
        1,
        1,
        &mut |candidate| {
            assert!(!candidate.exists());
            claimed.push(candidate.to_path_buf());
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(claimed.first(), Some(&path));
    assert_eq!(claimed.len(), 2);
    assert!(claimed[1]
        .extension()
        .is_some_and(|value| value == "partial"));
    assert!(!claimed[1].exists());
    assert_eq!(resource.path, path.to_string_lossy().as_ref());
    assert_eq!(std::fs::read(path).unwrap(), b"png");
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejected_ownership_prevents_resource_creation() {
    let directory =
        std::env::temp_dir().join(format!("document-ownership-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("resource.png");
    let storage = store_at(
        &path,
        hex::encode(Sha256::digest(b"png")),
        b"png",
        1,
        1,
        &mut |_| Err("inactive owner".into()),
    );
    assert!(storage.is_err());
    assert!(!path.exists());
    std::fs::remove_dir_all(directory).unwrap();
}
