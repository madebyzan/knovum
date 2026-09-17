use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use crate::ai::{capabilities::agent_artifacts, error::AiResult};

pub(super) struct TaskWorkspace {
    path: PathBuf,
}

impl TaskWorkspace {
    pub(super) fn new(app: &AppHandle) -> AiResult<Self> {
        let path = app
            .path()
            .app_data_dir()
            .map_err(|error| crate::ai::error::AiError::internal(error.to_string()))?
            .join("agent-workspaces")
            .join("tasks")
            .join(uuid::Uuid::new_v4().to_string());
        agent_artifacts::prepare(&path)?;
        Ok(Self { path })
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TaskWorkspace {
    fn drop(&mut self) {
        let _ = agent_artifacts::discard(&self.path);
    }
}
