use super::{checked_worker_root, RuntimeProfile};

#[test]
fn worker_directory_requires_the_selected_profile_entry() {
    let root = std::env::temp_dir().join(format!("knovum-worker-{}", uuid::Uuid::new_v4()));
    let entry = root.join(RuntimeProfile::Resources.entry());
    std::fs::create_dir_all(entry.parent().unwrap()).unwrap();
    std::fs::write(&entry, "").unwrap();

    assert_eq!(
        checked_worker_root(root.clone(), RuntimeProfile::Resources).unwrap(),
        root
    );
    assert!(checked_worker_root(root.clone(), RuntimeProfile::Agent).is_err());
    assert!(checked_worker_root(entry, RuntimeProfile::Resources).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
