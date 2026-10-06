use std::fs;
use tokio_util::sync::CancellationToken;

use super::super::ModelDescriptor;
use crate::local_resources::verification;

const DESCRIPTOR: ModelDescriptor = ModelDescriptor {
    snapshot: super::super::snapshots::SnapshotKind::ImageSegmentation,
    file: "model.bin",
    env_key: "KNOVUM_TEST_MODEL",
    sha256: "9372c470eeadd5ecd9c3c74c2b3cb633f8e2f2fad799250a0f70d652b6b825e4",
    download_size: 5,
};

#[test]
fn verified_receipt_replaces_repeated_content_hashing() {
    let root = std::env::temp_dir().join(format!("knovum-model-receipt-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let model = root.join(DESCRIPTOR.file);
    fs::write(&model, b"model").unwrap();

    assert!(verification::verify_and_record_cancellable(
        &model,
        DESCRIPTOR.sha256,
        &CancellationToken::new(),
        "Model verification cancelled",
    )
    .unwrap());
    let metadata = fs::metadata(&model).unwrap();
    assert!(verification::receipt_matches(&model, DESCRIPTOR.sha256, &metadata).unwrap());

    fs::remove_file(root.join("model.bin.verified.json")).unwrap();
    assert!(!verification::receipt_matches(&model, DESCRIPTOR.sha256, &metadata).unwrap());
    fs::remove_file(model).unwrap();
    fs::remove_dir(root).unwrap();
}
