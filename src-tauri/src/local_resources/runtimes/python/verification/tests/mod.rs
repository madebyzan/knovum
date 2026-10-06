use super::*;
use crate::local_resources::runtimes::python::descriptor::Descriptor;
use serde_json::json;

struct Fixture {
    root: PathBuf,
    bytes: Vec<u8>,
    descriptor: Descriptor,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "knovum-runtime-verification-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(root.join("packages")).unwrap();
        let executable = if cfg!(windows) {
            "python.exe"
        } else {
            "bin/python3"
        };
        let mut files = Vec::new();
        for (name, bytes) in [
            (executable, "interpreter"),
            ("runtime-requirements.txt", "requirements"),
            ("packages/dependency.py", "dependency"),
        ] {
            let path = root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, bytes).unwrap();
            files.push(
                json!({"path": name, "size": bytes.len(), "sha256": digest(bytes.as_bytes())}),
            );
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(root.join(executable), fs::Permissions::from_mode(0o755)).unwrap();
        }
        let requirements = digest(b"requirements");
        let bytes = serde_json::to_vec(&json!({"schemaVersion": 2, "runtimeId": "cpython", "version": "3.11.17",
            "dependencyRequirementsSha256": requirements, "voxcpmCommit": "a".repeat(40),
            "target": crate::local_resources::tools::platform_directory(), "executable": executable, "packageRoot": "packages", "files": files})).unwrap();
        fs::write(root.join(MANIFEST_NAME), &bytes).unwrap();
        let descriptor = Descriptor {
            schema_version: 1,
            target: crate::local_resources::tools::platform_directory().into(),
            version: "3.11.17".into(),
            dependency_requirements_sha256: requirements,
            voxcpm_commit: "a".repeat(40),
            sources: crate::local_resources::runtimes::python::descriptor::RuntimeSources {
                global: "https://example.test/runtime.tar.gz".into(),
                china: "https://china.example.test/runtime.tar.gz".into(),
            },
            size: 1,
            sha256: "b".repeat(64),
            manifest_sha256: digest(&bytes),
        };
        Self {
            root,
            bytes,
            descriptor,
        }
    }
    fn verify(&self, cancellation: &CancellationToken) -> Result<(PathBuf, PathBuf), String> {
        verify_runtime(
            &self.root.join(MANIFEST_NAME),
            &self.bytes,
            digest(&self.bytes),
            cancellation,
            Some(&self.descriptor),
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

#[test]
fn accepts_dependency_only_runtime_without_worker_or_build_evidence() {
    let fixture = Fixture::new();
    let (_, packages) = fixture.verify(&CancellationToken::new()).unwrap();
    assert_eq!(packages, fixture.root.join("packages"));
}

#[test]
fn rejects_recomputed_manifest_from_another_publication() {
    let mut fixture = Fixture::new();
    let mut manifest: serde_json::Value = serde_json::from_slice(&fixture.bytes).unwrap();
    manifest["voxcpmCommit"] = json!("c".repeat(40));
    fixture.bytes = serde_json::to_vec(&manifest).unwrap();
    assert!(fixture
        .verify(&CancellationToken::new())
        .unwrap_err()
        .contains("trusted descriptor"));
}

#[test]
fn rejects_changed_and_undeclared_dependency_files() {
    let fixture = Fixture::new();
    fs::write(fixture.root.join("packages/dependency.py"), "corrupt").unwrap();
    assert!(fixture
        .verify(&CancellationToken::new())
        .unwrap_err()
        .contains("file mismatch"));
    fs::write(fixture.root.join("packages/dependency.py"), "dependency").unwrap();
    fs::write(fixture.root.join("packages/extra.py"), "undeclared").unwrap();
    assert!(fixture
        .verify(&CancellationToken::new())
        .unwrap_err()
        .contains("undeclared file"));
}

#[test]
fn cancellation_interrupts_runtime_integrity_verification() {
    let fixture = Fixture::new();
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    assert_eq!(
        fixture.verify(&cancellation).unwrap_err(),
        crate::local_resources::preparation::RESOURCE_PREPARATION_CANCELLED
    );
}
