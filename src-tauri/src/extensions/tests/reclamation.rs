use super::collect_content;
use std::{collections::BTreeSet, fs};

#[test]
fn reclamation_only_removes_unretained_directories() {
    let root = std::env::temp_dir().join(format!("knovum-reclamation-{}", uuid::Uuid::new_v4()));
    let retained = BTreeSet::from(["plugins/installed/revisions/current".to_owned()]);
    for directory in [
        "plugins/installed/revisions/current",
        "plugins/installed/revisions/retired",
        "skills/deleted/revisions/retired",
        "plugin-data/installed",
        "plugin-data/deleted",
    ] {
        fs::create_dir_all(root.join(directory)).unwrap();
    }
    for file in [
        "plugins/.DS_Store",
        "plugins/installed/revisions/.DS_Store",
        "plugins/installed/revisions/readme.txt",
        "skills/.DS_Store",
        "plugin-data/.DS_Store",
    ] {
        fs::write(root.join(file), b"unmanaged file").unwrap();
    }
    let collection = collect_content(&root, &retained, BTreeSet::new());
    assert!(collection.is_ok(), "{collection:?}");
    for directory in [
        "plugins/installed/revisions/current",
        "plugin-data/installed",
    ] {
        assert!(root.join(directory).is_dir(), "{directory}");
    }
    for directory in [
        "plugins/installed/revisions/retired",
        "skills/deleted",
        "plugin-data/deleted",
    ] {
        assert!(!root.join(directory).exists(), "{directory}");
    }
    assert_eq!(
        fs::read(root.join("plugins/installed/revisions/.DS_Store")).unwrap(),
        b"unmanaged file"
    );
    assert!(root
        .join("plugins/installed/revisions/readme.txt")
        .is_file());
    assert!(root.join("plugin-data/.DS_Store").is_file());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reclamation_rejects_a_file_at_a_required_directory_path() {
    let root = std::env::temp_dir().join(format!(
        "knovum-reclamation-invalid-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("plugins"), b"not a directory").unwrap();
    assert!(collect_content(&root, &BTreeSet::new(), BTreeSet::new()).is_err());
    assert_eq!(fs::read(root.join("plugins")).unwrap(), b"not a directory");
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn reclamation_does_not_traverse_revision_or_plugin_data_symlinks() {
    let root =
        std::env::temp_dir().join(format!("knovum-reclamation-links-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(root.join("plugins/installed/revisions")).unwrap();
    fs::create_dir_all(root.join("plugin-data")).unwrap();
    fs::create_dir(root.join("outside")).unwrap();
    fs::write(root.join("outside/keep.txt"), b"keep").unwrap();
    for link in [
        "plugins/installed/revisions/link",
        "plugin-data/link",
        "plugins/link",
    ] {
        std::os::unix::fs::symlink(root.join("outside"), root.join(link)).unwrap();
    }
    collect_content(&root, &BTreeSet::new(), BTreeSet::new()).unwrap();
    assert_eq!(fs::read(root.join("outside/keep.txt")).unwrap(), b"keep");
    assert!(
        fs::symlink_metadata(root.join("plugins/installed/revisions/link"))
            .unwrap()
            .is_symlink()
    );
    assert!(fs::symlink_metadata(root.join("plugin-data/link"))
        .unwrap()
        .is_symlink());
    fs::remove_dir_all(root).unwrap();
}
