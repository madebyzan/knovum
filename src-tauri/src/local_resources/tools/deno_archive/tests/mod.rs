use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use tokio_util::sync::CancellationToken;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

use super::*;

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("knovum-deno-archive-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn extracts_the_single_expected_binary() {
    let directory = TestDirectory::new();
    let archive = directory.0.join("deno.zip");
    let binary = directory.0.join("deno");
    write_archive(&archive, &[("deno", b"deno-binary")]);

    extract_blocking(&archive, &binary, "deno", &CancellationToken::new()).unwrap();

    assert_eq!(fs::read(binary).unwrap(), b"deno-binary");
}

#[test]
fn rejects_parent_directory_entries() {
    let directory = TestDirectory::new();
    let archive = directory.0.join("deno.zip");
    let binary = directory.0.join("deno");
    write_archive(&archive, &[("../deno", b"deno-binary")]);

    let error = extract_blocking(&archive, &binary, "deno", &CancellationToken::new())
        .expect_err("parent directory entry must be rejected");

    assert!(error.contains("unsafe path"));
    assert!(!binary.exists());
}

#[test]
fn rejects_archives_with_additional_entries() {
    let directory = TestDirectory::new();
    let archive = directory.0.join("deno.zip");
    let binary = directory.0.join("deno");
    write_archive(
        &archive,
        &[("deno", b"deno-binary"), ("README", b"unexpected")],
    );

    let error = extract_blocking(&archive, &binary, "deno", &CancellationToken::new())
        .expect_err("additional archive entries must be rejected");

    assert!(error.contains("exactly one executable"));
    assert!(!binary.exists());
}

#[test]
fn cancellation_does_not_create_output() {
    let directory = TestDirectory::new();
    let archive = directory.0.join("deno.zip");
    let binary = directory.0.join("deno");
    write_archive(&archive, &[("deno", b"deno-binary")]);
    let cancellation = CancellationToken::new();
    cancellation.cancel();

    let error = extract_blocking(&archive, &binary, "deno", &cancellation)
        .expect_err("cancelled extraction must stop before writing");

    assert_eq!(
        error,
        super::super::super::preparation::RESOURCE_PREPARATION_CANCELLED
    );
    assert!(!binary.exists());
}

fn write_archive(path: &Path, entries: &[(&str, &[u8])]) {
    let file = fs::File::create(path).unwrap();
    let mut archive = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for (name, bytes) in entries {
        archive.start_file(*name, options).unwrap();
        archive.write_all(bytes).unwrap();
    }
    archive.finish().unwrap();
}
