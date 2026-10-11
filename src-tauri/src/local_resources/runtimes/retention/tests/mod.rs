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
    let orphan = root.join(format!("{}.lease", "d".repeat(64)));
    fs::write(&orphan, []).unwrap();
    let lease = acquire(&running).unwrap();
    let other_owner = lease.clone();
    let duplicated_file = lease.0.try_clone().unwrap();
    retire_unused(&current).unwrap();
    assert!(current.is_dir());
    assert!(running.is_dir());
    assert!(!unused.exists());
    assert!(!orphan.exists());
    assert!(!unused.with_extension("lease").exists());
    drop(lease);
    retire_unused(&current).unwrap();
    assert!(running.is_dir());
    drop(other_owner);
    retire_unused(&current).unwrap();
    assert!(!running.exists());
    assert!(!running.with_extension("lease").exists());
    drop(duplicated_file);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn installation_waits_for_active_runtime_and_hands_lease_to_workers() {
    let root = std::env::temp_dir().join(format!("knovum-runtime-reset-{}", uuid::Uuid::new_v4()));
    let directory = root.join("a".repeat(64));
    fs::create_dir_all(&directory).unwrap();
    let running = acquire(&directory).unwrap();
    let (acquired_tx, acquired_rx) = std::sync::mpsc::channel();
    let contender = directory.clone();
    let installer = std::thread::spawn(move || {
        let lease = loop {
            if let Some(lease) = try_acquire_exclusive(&contender).unwrap() {
                break lease;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        acquired_tx.send(lease).unwrap();
    });
    assert!(acquired_rx
        .recv_timeout(std::time::Duration::from_millis(100))
        .is_err());
    drop(running);
    let installation = acquired_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    let worker = installation.into_shared().unwrap();
    let current = root.join("b".repeat(64));
    retire_unused(&current).unwrap();
    assert!(directory.is_dir());
    drop(worker);
    retire_unused(&current).unwrap();
    assert!(!directory.exists());
    assert!(!directory.with_extension("lease").exists());
    installer.join().unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn publication_moves_one_validated_directory_and_keeps_it_leased() {
    let root =
        std::env::temp_dir().join(format!("knovum-runtime-publish-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let destination = root.join("a".repeat(64));
    let prepared = root.join(".prepared");
    fs::create_dir(&prepared).unwrap();
    fs::write(prepared.join("runtime-manifest.json"), b"validated").unwrap();
    let installation = try_acquire_exclusive(&destination).unwrap().unwrap();
    let worker = installation
        .publish_directory(&prepared, &destination)
        .unwrap();
    assert!(!prepared.exists());
    assert_eq!(
        fs::read(destination.join("runtime-manifest.json")).unwrap(),
        b"validated"
    );
    let current = root.join("b".repeat(64));
    retire_unused(&current).unwrap();
    assert!(destination.is_dir());
    drop(worker);
    retire_unused(&current).unwrap();
    assert!(!destination.exists());
    fs::remove_dir_all(root).unwrap();
}
