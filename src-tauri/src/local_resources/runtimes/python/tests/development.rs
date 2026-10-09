use super::*;
use crate::local_resources::runtimes::python::verification::tests::Fixture;

#[test]
fn producer_scope_reclaims_only_its_owned_staging() {
    let root = std::env::temp_dir().join(format!("knovum-producer-test-{}", uuid::Uuid::new_v4()));
    let staging = Staging::create(root.clone()).unwrap();
    let path = staging.0.clone();
    std::fs::create_dir(path.join("unfinished")).unwrap();
    std::fs::write(path.join("unfinished/weights"), b"partial download").unwrap();
    std::fs::write(root.join("other-job"), b"keep").unwrap();
    drop(staging);
    assert!(!path.exists());
    assert_eq!(std::fs::read(root.join("other-job")).unwrap(), b"keep");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn cancelled_verification_keeps_the_owned_runtime_directory() {
    let fixture = Fixture::new();
    let profile = RuntimeProfile::TorchInference;
    let mut manifest: serde_json::Value = serde_json::from_slice(&fixture.bytes).unwrap();
    manifest["dependencyLockSha256"] =
        serde_json::json!(super::super::verification::digest(profile.lock_bytes()));
    let path = fixture.root.join(super::super::MANIFEST_NAME);
    std::fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    assert!(matches!(
        prepare_identity(&fixture.root, &path, profile, &cancellation),
        Err(RuntimeError::Cancelled)
    ));
    assert!(path.is_file());
    assert!(fixture.root.join("packages/dependency.py").is_file());
}

#[cfg(unix)]
#[test]
fn metadata_io_failure_keeps_the_owned_runtime_directory() {
    let fixture = Fixture::new();
    let path = fixture
        .root
        .join("packages/dependency.py")
        .join(super::super::MANIFEST_NAME);
    assert!(matches!(
        prepare_identity(
            &fixture.root,
            &path,
            RuntimeProfile::TorchInference,
            &CancellationToken::new()
        ),
        Err(RuntimeError::Infrastructure(_))
    ));
    assert!(fixture.root.join("packages/dependency.py").is_file());
}
