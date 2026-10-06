use super::super::snapshots::SnapshotFile;
use super::*;

const FILE: SnapshotFile = SnapshotFile {
    path: "model.bin",
    sha256: "9372c470eeadd5ecd9c3c74c2b3cb633f8e2f2fad799250a0f70d652b6b825e4",
    size: 5,
};

struct Directory(std::path::PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("knovum-snapshot-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn records_receipts_only_for_verified_snapshot_content() {
    let directory = Directory::new();
    let path = directory.0.join(FILE.path);
    fs::write(&path, b"model").unwrap();
    verify_and_record(&directory.0, &[FILE], &CancellationToken::new()).unwrap();
    assert!(
        verification::receipt_matches(&path, FILE.sha256, &fs::metadata(&path).unwrap()).unwrap()
    );
}

#[test]
fn rejects_and_removes_corrupt_download_for_next_sdk_attempt() {
    for bytes in [b"wrong".as_slice(), b"shorter".as_slice()] {
        let directory = Directory::new();
        let path = directory.0.join(FILE.path);
        fs::write(&path, bytes).unwrap();
        assert!(verify_and_record(&directory.0, &[FILE], &CancellationToken::new()).is_err());
        assert!(!path.exists());
    }
}

#[test]
fn cancellation_preserves_staging_and_prevents_verification() {
    let directory = Directory::new();
    let path = directory.0.join(FILE.path);
    fs::write(&path, b"model").unwrap();
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    assert_eq!(
        verify_and_record(&directory.0, &[FILE], &cancellation).unwrap_err(),
        RESOURCE_PREPARATION_CANCELLED
    );
    assert!(path.exists());
    assert!(
        !verification::receipt_matches(&path, FILE.sha256, &fs::metadata(&path).unwrap()).unwrap()
    );
}
