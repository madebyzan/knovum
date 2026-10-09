use std::{
    fs,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use super::{archive, planning::AssetPlan};
use crate::spaces::package::AssetIndex;

#[test]
fn cancellation_between_asset_chunks_retains_cancelled_outcome() {
    let root = std::env::temp_dir().join(format!("knovum-package-cancel-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).expect("temporary directory");
    let source = root.join("source.bin");
    let archive_path = root.join("staging.zip");
    let bytes = 3 * 1024 * 1024;
    fs::write(&source, vec![7_u8; bytes]).expect("asset fixture");
    let token = CancellationToken::new();
    let cancellation = token.clone();
    let updates = Arc::new(AtomicUsize::new(0));
    let observed = updates.clone();
    let progress = Channel::new(move |_| {
        // The manifest is first; cancel immediately after the first asset chunk.
        if observed.fetch_add(1, Ordering::SeqCst) == 1 {
            cancellation.cancel();
        }
        Ok(())
    });
    let plan = AssetPlan {
        token: "asset".into(),
        source,
        bytes: bytes as u64,
        index: AssetIndex {
            token: "asset".into(),
            name: "source.bin".into(),
            directory: false,
            source_revision: None,
        },
    };
    let outcome = archive::write(
        fs::File::create(&archive_path).expect("staging file"),
        "{}",
        b"[]",
        &[plan],
        &progress,
        bytes as u64 + 4,
        &token,
    );
    let error = outcome.expect_err("cancelled archive must not complete");
    assert_eq!(error.code, "cancelled");
    assert_eq!(updates.load(Ordering::SeqCst), 2);
    assert!(fs::metadata(&archive_path).expect("staging metadata").len() < bytes as u64);
    fs::remove_dir_all(root).expect("fixture cleanup");
}
