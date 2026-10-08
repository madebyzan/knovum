use super::*;

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
