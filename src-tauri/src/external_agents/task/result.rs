use std::{fs, io::Read, path::Path};

use crate::ai::{
    capabilities::agent_artifacts,
    error::{AiError, AiErrorCode, AiResult},
};

pub(super) fn read(root: &Path, name: &str, limit: u64) -> AiResult<Vec<u8>> {
    let invalid = |message| AiError::new(AiErrorCode::InvalidResponse, message);
    let path = agent_artifacts::resolve(root, name).map_err(|e| {
        invalid(format!(
            "Desktop Agent did not submit a valid task result: {e}"
        ))
    })?;
    let file = fs::File::open(&path).map_err(|e| invalid(e.to_string()))?;
    let metadata = file.metadata().map_err(|e| invalid(e.to_string()))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > limit {
        return Err(invalid(
            "Desktop Agent result is not a nonempty regular file within its task byte budget"
                .into(),
        ));
    }
    let revision = crate::file_inspection::content_revision(&path).map_err(&invalid)?;
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| invalid(e.to_string()))?;
    if bytes.is_empty() || bytes.len() as u64 > limit {
        return Err(invalid(
            "Desktop Agent result is empty or exceeds its task byte budget".into(),
        ));
    }
    use sha2::{Digest, Sha256};
    if format!("sha256:{:x}", Sha256::digest(&bytes)) != revision {
        return Err(invalid("Desktop Agent result changed while reading".into()));
    }
    if fs::metadata(&path)
        .map_err(|e| invalid(e.to_string()))?
        .len()
        != bytes.len() as u64
    {
        return Err(invalid(
            "Desktop Agent result size changed while reading".into(),
        ));
    }
    crate::file_inspection::verify_content_revision(&path, &revision).map_err(invalid)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn workspace() -> super::super::TaskWorkspace {
        let root =
            std::env::temp_dir().join(format!("knovum-task-result-{}", uuid::Uuid::new_v4()));
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
}
