
use super::*;
fn workspace() -> super::super::TaskWorkspace {
    let root = std::env::temp_dir().join(format!("knovum-task-result-{}", uuid::Uuid::new_v4()));
    agent_artifacts::prepare(&root).unwrap();
    super::super::TaskWorkspace(root)
}
#[test]
fn only_the_valid_named_file_is_a_deliverable_and_workspaces_are_cleaned() {
    let workspace = workspace();
    assert!(read(&workspace.0, "result.txt", 1024).is_err());
    fs::write(
        workspace.0.join("unrelated.txt"),
        "ordinary chat is not a result",
    )
    .unwrap();
    assert!(read(&workspace.0, "result.txt", 1024).is_err());
    fs::write(workspace.0.join("result.txt"), "strict output").unwrap();
    assert_eq!(
        read(&workspace.0, "result.txt", 1024).unwrap(),
        b"strict output"
    );
    let path = workspace.0.clone();
    drop(workspace);
    assert!(!path.exists());
}
#[test]
fn empty_oversized_and_escaping_results_are_rejected() {
    let workspace = workspace();
    fs::write(workspace.0.join("result.txt"), b"").unwrap();
    assert!(read(&workspace.0, "result.txt", 1024).is_err());
    fs::write(workspace.0.join("result.txt"), [1u8; 33]).unwrap();
    assert!(read(&workspace.0, "result.txt", 32).is_err());
    assert!(read(&workspace.0, "../result.txt", 1024).is_err());
}
#[cfg(unix)]
#[test]
fn a_symlink_is_not_an_authorized_result_file() {
    let workspace = workspace();
    fs::write(workspace.0.join("actual.txt"), "output").unwrap();
    std::os::unix::fs::symlink(
        workspace.0.join("actual.txt"),
        workspace.0.join("result.txt"),
    )
    .unwrap();
    assert!(read(&workspace.0, "result.txt", 1024).is_err());
}
