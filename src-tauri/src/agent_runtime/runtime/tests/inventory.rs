use super::super::inventory::verify;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "knovum-agent-inventory-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("knovum-agent"), b"agent").unwrap();
        Self(root)
    }
    fn seal(&self, entries: serde_json::Value) -> Vec<u8> {
        let bytes = serde_json::to_vec(&json!({"schemaVersion": 1,
            "target": crate::local_resources::tools::platform_directory(), "files": entries}))
        .unwrap();
        fs::write(self.0.join("agent-manifest.json"), &bytes).unwrap();
        bytes
    }
    fn executable(&self) -> serde_json::Value {
        json!({"path": "knovum-agent", "size": 5, "sha256": hex::encode(Sha256::digest(b"agent"))})
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn rejects_changed_missing_and_unlisted_resources() {
    let fixture = Fixture::new();
    let trusted = fixture.seal(json!([fixture.executable()]));
    verify(&fixture.0, &trusted).unwrap();
    fs::write(fixture.0.join("unexpected"), b"extra").unwrap();
    assert!(verify(&fixture.0, &trusted).is_err());
    fs::remove_file(fixture.0.join("unexpected")).unwrap();
    fs::write(fixture.0.join("knovum-agent"), b"other").unwrap();
    assert!(verify(&fixture.0, &trusted).is_err());
    fs::remove_file(fixture.0.join("knovum-agent")).unwrap();
    assert!(verify(&fixture.0, &trusted).is_err());
}

#[cfg(unix)]
#[test]
fn preserves_relative_links_and_rejects_an_escaping_target() {
    let fixture = Fixture::new();
    std::os::unix::fs::symlink("knovum-agent", fixture.0.join("alias")).unwrap();
    let trusted =
        fixture.seal(json!([fixture.executable(), {"path": "alias", "link": "knovum-agent"}]));
    verify(&fixture.0, &trusted).unwrap();
    fs::remove_file(fixture.0.join("alias")).unwrap();
    std::os::unix::fs::symlink("..", fixture.0.join("alias")).unwrap();
    assert!(verify(&fixture.0, &trusted).is_err());
}
