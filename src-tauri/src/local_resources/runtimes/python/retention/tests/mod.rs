use super::*;

#[test]
fn retirement_preserves_current_and_leased_runtimes() {
    let root =
        std::env::temp_dir().join(format!("knovum-runtime-retention-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let current = root.join("a".repeat(64));
    let running = root.join("b".repeat(64));
    let unused = root.join("c".repeat(64));
    for path in [&current, &running, &unused] {
        fs::create_dir(path).unwrap();
    }
    let lease = acquire(&running).unwrap();
    let other_owner = lease.clone();
    let duplicated_file = lease.0.try_clone().unwrap();
    retire_unused(&current).unwrap();
    assert!(current.is_dir());
    assert!(running.is_dir());
    assert!(!unused.exists());
    drop(lease);
    retire_unused(&current).unwrap();
    assert!(running.is_dir());
    drop(other_owner);
    retire_unused(&current).unwrap();
    assert!(!running.exists());
    drop(duplicated_file);
    fs::remove_dir_all(root).unwrap();
}
