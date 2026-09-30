use std::{fs, path::PathBuf};

use tauri::{AppHandle, Runtime};
use tauri_plugin_fs::FsExt;

use crate::ai::error::{AiError, AiResult};

pub(super) struct AgentAttachments(Vec<PathBuf>);

impl AgentAttachments {
    pub(super) fn from_authorized_assets(
        attachments: Vec<PathBuf>,
        authorize: impl Fn(&std::path::Path) -> AiResult<()>,
    ) -> AiResult<Self> {
        validate(attachments, authorize).map(Self)
    }

    pub(super) fn from_renderer<R: Runtime>(
        app: &AppHandle<R>,
        attachments: Vec<PathBuf>,
    ) -> AiResult<Self> {
        let scope = app.fs_scope();
        validate(attachments, |path| {
            if scope.is_allowed(path) {
                Ok(())
            } else {
                Err(AiError::invalid(
                    "Agent attachment was not selected or dropped into Knovum",
                ))
            }
        })
        .map(Self)
    }

    pub(super) fn into_paths(self) -> Vec<PathBuf> {
        self.0
    }
}

fn validate(
    attachments: Vec<PathBuf>,
    authorize: impl Fn(&std::path::Path) -> AiResult<()>,
) -> AiResult<Vec<PathBuf>> {
    attachments
        .into_iter()
        .map(|source| {
            authorize(&source)?;
            let metadata = fs::symlink_metadata(&source)
                .map_err(|error| AiError::invalid(format!("Cannot inspect attachment: {error}")))?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(AiError::invalid("Agent attachments must be regular files"));
            }
            let canonical = source
                .canonicalize()
                .map_err(|error| AiError::invalid(format!("Cannot open attachment: {error}")))?;
            authorize(&canonical)?;
            Ok(canonical)
        })
        .collect()
}
