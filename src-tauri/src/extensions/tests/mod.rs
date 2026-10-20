use super::content::{self, PackageFiles};

#[test]
fn portable_paths_reject_aliases_and_file_directory_collisions() {
    for paths in [
        vec!["A/file", "a/other"],
        vec!["a", "a/file"],
        vec!["é.txt", "e\u{301}.txt"],
        vec!["CON.txt"],
        vec!["a/../b"],
        vec!["a\\b"],
        vec!["a//b"],
        vec!["a<file"],
        vec!["a?file"],
        vec!["a\u{1f}file"],
    ] {
        let files: PackageFiles = paths
            .into_iter()
            .map(|path| (path.into(), vec![]))
            .collect();
        assert!(content::validate_files(&files).is_err());
    }
    assert!(content::validate_paths([("A", true), ("a/file", false)].into_iter()).is_err());
    assert!(content::validate_paths([("a", true), ("a/file", false)].into_iter()).is_ok());
}

#[test]
fn digest_frames_path_and_payload_lengths() {
    let first = [("a".into(), b"bc".to_vec())].into_iter().collect();
    let second = [("ab".into(), b"c".to_vec())].into_iter().collect();
    assert_ne!(
        content::files_digest(&first),
        content::files_digest(&second)
    );
}

#[test]
fn immutable_publish_never_overwrites_a_corrupt_revision() {
    let root = std::env::temp_dir().join(format!("knovum-content-test-{}", uuid::Uuid::new_v4()));
    let files = [("SKILL.md".into(), b"reviewed".to_vec())]
        .into_iter()
        .collect();
    content::publish(&root, &files).unwrap();
    std::fs::write(root.join("SKILL.md"), b"changed").unwrap();
    assert!(content::publish(&root, &files).is_err());
    assert_eq!(std::fs::read(root.join("SKILL.md")).unwrap(), b"changed");
    std::fs::remove_dir_all(root).unwrap();
}
