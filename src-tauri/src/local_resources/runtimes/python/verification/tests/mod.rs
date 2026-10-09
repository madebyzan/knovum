use super::*;
use crate::local_resources::runtimes::python::descriptor::Descriptor;
use serde_json::json;

pub(in crate::local_resources::runtimes::python) struct Fixture {
    pub(in crate::local_resources::runtimes::python) root: PathBuf,
    pub(in crate::local_resources::runtimes::python) bytes: Vec<u8>,
    descriptor: Descriptor,
}
impl Fixture {
    pub(in crate::local_resources::runtimes::python) fn new() -> Self {
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
        let bytes = serde_json::to_vec(&json!({"schemaVersion": 1, "runtimeId": "cpython", "version": "3.11.17",
            "dependencyRequirementsSha256": requirements, "profile":"torch-inference", "dependencyLockSha256":"a".repeat(64),
            "target": crate::local_resources::tools::platform_directory(), "executable": executable, "packageRoot": "packages", "files": files})).unwrap();
        fs::write(root.join(MANIFEST_NAME), &bytes).unwrap();
        let descriptor = Descriptor {
            schema_version: 1,
            target: crate::local_resources::tools::platform_directory().into(),
            version: "3.11.17".into(),
            dependency_requirements_sha256: requirements,
            profile: crate::local_resources::runtimes::python::RuntimeProfile::TorchInference,
            dependency_lock_sha256: "a".repeat(64),
            archive_url: "https://example.test/runtime.tar.gz".into(),
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
    fn verify(&self, cancellation: &CancellationToken) -> Result<(PathBuf, PathBuf), RuntimeError> {
        verify_runtime(
            &self.root.join(MANIFEST_NAME),
            &self.bytes,
            digest(&self.bytes),
            cancellation,
            Some(&self.descriptor),
            crate::local_resources::runtimes::python::RuntimeProfile::TorchInference,
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        crate::local_resources::filesystem::remove_directory(&self.root).unwrap();
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
    manifest["dependencyLockSha256"] = json!("c".repeat(64));
    fixture.bytes = serde_json::to_vec(&manifest).unwrap();
    assert!(fixture
        .verify(&CancellationToken::new())
        .unwrap_err()
        .to_string()
        .contains("trusted descriptor"));
}

#[cfg(debug_assertions)]
#[test]
fn development_runtime_must_match_the_current_capability_lock() {
    let fixture = Fixture::new();
    let profile = crate::local_resources::runtimes::python::RuntimeProfile::TorchInference;
    let verify = |bytes: &[u8]| {
        verify_runtime(
            &fixture.root.join(MANIFEST_NAME),
            bytes,
            digest(bytes),
            &CancellationToken::new(),
            None,
            profile,
        )
    };
    assert!(verify(&fixture.bytes)
        .unwrap_err()
        .to_string()
        .contains("dependency lock"));
    let mut manifest: serde_json::Value = serde_json::from_slice(&fixture.bytes).unwrap();
    manifest["dependencyLockSha256"] = json!(digest(profile.lock_bytes()));
    verify(&serde_json::to_vec(&manifest).unwrap()).unwrap();
}

#[test]
fn rejects_changed_and_undeclared_dependency_files() {
    let fixture = Fixture::new();
    fs::write(fixture.root.join("packages/dependency.py"), "corrupt").unwrap();
    assert!(matches!(
        fixture.verify(&CancellationToken::new()),
        Err(RuntimeError::InvalidPublication(_))
    ));
    fs::write(fixture.root.join("packages/dependency.py"), "dependency").unwrap();
    fs::write(fixture.root.join("packages/extra.py"), "undeclared").unwrap();
    assert!(matches!(
        fixture.verify(&CancellationToken::new()),
        Err(RuntimeError::InvalidPublication(_))
    ));
}

#[test]
fn cancellation_interrupts_runtime_integrity_verification() {
    let fixture = Fixture::new();
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    assert!(matches!(
        fixture.verify(&cancellation),
        Err(RuntimeError::Cancelled)
    ));
}

#[test]
fn missing_manifest_is_an_invalid_publication() {
    let fixture = Fixture::new();
    fs::remove_file(fixture.root.join(MANIFEST_NAME)).unwrap();
    assert!(matches!(
        read_manifest(&fixture.root.join(MANIFEST_NAME)),
        Err(RuntimeError::InvalidPublication(_))
    ));
    assert!(!manifest_exists(&fixture.root.join(MANIFEST_NAME)).unwrap());
}

#[cfg(unix)]
#[test]
fn metadata_io_failure_does_not_authorize_runtime_reset() {
    let fixture = Fixture::new();
    let path = fixture
        .root
        .join("packages/dependency.py")
        .join(MANIFEST_NAME);
    assert!(matches!(
        read_manifest(&path),
        Err(RuntimeError::Infrastructure(_))
    ));
    assert!(matches!(
        manifest_exists(&path),
        Err(RuntimeError::Infrastructure(_))
    ));
}
