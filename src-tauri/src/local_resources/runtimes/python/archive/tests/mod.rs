use super::*;
use flate2::{write::GzEncoder, Compression};
use std::path::PathBuf;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "knovum-runtime-extraction-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn archive(&self, entries: &[(&str, tar::EntryType)]) -> PathBuf {
        let path = self.0.join("runtime.tar.gz");
        let encoder = GzEncoder::new(fs::File::create(&path).unwrap(), Compression::default());
        let mut builder = tar::Builder::new(encoder);
        for (name, kind) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(*kind);
            header.set_mode(0o755);
            header.set_size(0);
            header.set_cksum();
            builder
                .append_data(&mut header, name, std::io::empty())
                .unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap();
        path
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn extracts_regular_files_and_directory_entries() {
    let fixture = Fixture::new();
    let archive = fixture.archive(&[
        ("packages/", tar::EntryType::Directory),
        ("packages/module.py", tar::EntryType::Regular),
    ]);
    let destination = fixture.0.join("unpacked");
    extract(&archive, &destination, &CancellationToken::new()).unwrap();
    assert!(destination.join("packages/module.py").is_file());
}

#[test]
fn rejects_unsafe_paths() {
    for path in [
        "",
        "/packages",
        "../module.py",
        "packages/../module.py",
        "./module.py",
        "packages//module.py",
        "packages//",
        "packages\\module.py",
        "C:/module.py",
    ] {
        assert!(!safe_path(Path::new(path)), "accepted {path:?}");
    }
    assert!(safe_path(Path::new("packages/")));
    assert!(safe_path(Path::new("packages/module.py")));
}

#[test]
fn rejects_links_and_duplicate_files() {
    for entries in [
        vec![("linked", tar::EntryType::Symlink)],
        vec![
            ("duplicate", tar::EntryType::Regular),
            ("duplicate", tar::EntryType::Regular),
        ],
    ] {
        let fixture = Fixture::new();
        let archive = fixture.archive(&entries);
        assert!(extract(
            &archive,
            &fixture.0.join("unpacked"),
            &CancellationToken::new()
        )
        .is_err());
    }
}

#[test]
fn cancelled_extraction_never_writes_runtime_files() {
    let fixture = Fixture::new();
    let archive = fixture.archive(&[("module.py", tar::EntryType::Regular)]);
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let destination = fixture.0.join("unpacked");
    assert_eq!(
        extract(&archive, &destination, &cancellation).unwrap_err(),
        RESOURCE_PREPARATION_CANCELLED
    );
    assert!(!destination.exists());
}
