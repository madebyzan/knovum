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
