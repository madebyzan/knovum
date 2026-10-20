use super::super::{archive, directory_files};
use std::io::{Cursor, Write};
use tokio_util::sync::CancellationToken;
use zip::{write::SimpleFileOptions, ZipWriter};

fn package(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    for (path, bytes) in entries {
        archive
            .start_file(*path, SimpleFileOptions::default())
            .unwrap();
        archive.write_all(bytes).unwrap();
    }
    archive.finish().unwrap().into_inner()
}

#[test]
fn archive_rejects_traversal_case_aliases_and_file_directory_collisions() {
    for entries in [
        vec![("../outside", &b"text"[..])],
        vec![("/absolute", &b"text"[..])],
        vec![("A/file", &b"text"[..]), ("a/other", &b"text"[..])],
        vec![("file", &b"text"[..]), ("file/child", &b"text"[..])],
    ] {
        assert!(archive::extract(package(&entries), &CancellationToken::new()).is_err());
    }
}

#[test]
fn archive_preserves_binary_resources_and_rejects_symlinks_and_oversized_files() {
    let binary = (0..=255).collect::<Vec<u8>>();
    let files = archive::extract(
        package(&[("reference.bin", &binary)]),
        &CancellationToken::new(),
    )
    .unwrap();
    assert_eq!(files["reference.bin"], binary);
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    archive
        .add_symlink("link", "../outside", SimpleFileOptions::default())
        .unwrap();
    assert_eq!(
        archive::extract(
            archive.finish().unwrap().into_inner(),
            &CancellationToken::new()
        )
        .unwrap_err()
        .message,
        "package-special-file-not-supported"
    );
    let oversized = vec![0; crate::extensions::content::MAX_FILE_BYTES + 1];
    assert_eq!(
        archive::extract(package(&[("large", &oversized)]), &CancellationToken::new())
            .unwrap_err()
            .message,
        "package-file-size-limit"
    );
}

#[test]
fn cancelled_archive_extraction_does_not_read_content() {
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let error = archive::extract(package(&[("resource", b"content")]), &cancellation).unwrap_err();
    assert_eq!(error.code, "cancelled");
    assert_eq!(error.message, "Plugin source reading was cancelled");
}

#[test]
fn cancelled_directory_read_returns_a_stable_error_code() {
    let root = std::env::temp_dir().join(format!("knovum-source-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("resource"), b"content").unwrap();
    let cancellation = CancellationToken::new();
    cancellation.cancel();

    let error = directory_files(&root, &cancellation).unwrap_err();

    assert_eq!(error.code, "cancelled");
    assert_eq!(error.message, "Plugin source reading was cancelled");
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn local_sources_reject_symbolic_and_hard_links() {
    let root = std::env::temp_dir().join(format!("knovum-source-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("original"), b"resource").unwrap();
    std::os::unix::fs::symlink("original", root.join("alias")).unwrap();
    assert_eq!(
        directory_files(&root, &CancellationToken::new())
            .unwrap_err()
            .message,
        "package-special-file-not-supported"
    );
    std::fs::remove_file(root.join("alias")).unwrap();
    std::fs::hard_link(root.join("original"), root.join("alias")).unwrap();
    assert_eq!(
        directory_files(&root, &CancellationToken::new())
            .unwrap_err()
            .message,
        "package-hardlink-not-supported"
    );
    std::fs::remove_dir_all(root).unwrap();
}
