use sha2::{Digest, Sha256};

use super::source_snapshot::copy;

#[test]
fn snapshot_copy_preserves_the_exact_frozen_length_and_digest() {
    let root = std::env::temp_dir().join(format!("knovum-voice-snapshot-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    let source = root.join("source.wav");
    let target = root.join("snapshot.input");
    let content = b"fixed voice reference";
    std::fs::write(&source, content).unwrap();
    let revision = format!("sha256:{}", hex::encode(Sha256::digest(content)));

    copy(
        &source,
        &target,
        u64::try_from(content.len()).unwrap(),
        &revision,
    )
    .unwrap();

    assert_eq!(std::fs::read(&target).unwrap(), content);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn snapshot_copy_rejects_content_beyond_the_frozen_length() {
    let root = std::env::temp_dir().join(format!(
        "knovum-voice-snapshot-growth-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir(&root).unwrap();
    let source = root.join("source.wav");
    let target = root.join("snapshot.input");
    let content = b"frozen+growth";
    let frozen = &content[..6];
    std::fs::write(&source, content).unwrap();
    let revision = format!("sha256:{}", hex::encode(Sha256::digest(frozen)));

    let error = copy(
        &source,
        &target,
        u64::try_from(frozen.len()).unwrap(),
        &revision,
    )
    .unwrap_err();

    assert!(error.contains("grew"));
    assert_eq!(std::fs::metadata(&target).unwrap().len(), 6);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_metadata_allows_long_media_without_copying_the_entire_reference() {
    let path = std::env::temp_dir().join(format!("knovum-long-reference-{}", uuid::Uuid::new_v4()));
    let file = std::fs::File::create(&path).unwrap();
    file.set_len(129 * 1024 * 1024).unwrap();
    super::source_snapshot::validate_source_metadata(&file.metadata().unwrap()).unwrap();
    drop(file);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn frozen_clip_contract_requires_duration_and_rejects_missing_pre_release_metadata() {
    let raw = serde_json::json!({"assetPath":"/managed/clip.input","sourceRevision":format!("sha256:{}", "a".repeat(64)),"byteLength":160044});
    assert!(serde_json::from_value::<super::ReferenceSourceSnapshot>(raw.clone()).is_err());
    let mut complete = raw;
    complete["durationMs"] = serde_json::json!(5000);
    let snapshot: super::ReferenceSourceSnapshot = serde_json::from_value(complete).unwrap();
    assert_eq!(snapshot.duration_ms, 5000);
}
