use std::{fs, path::PathBuf};

use tauri::{AppHandle, Runtime};
use tauri_plugin_fs::FsExt;

use crate::app_error::{AppError, AppResult};

pub(super) struct AgentAttachments(Vec<PathBuf>);

impl AgentAttachments {
    pub(super) fn from_authorized_assets(
        attachments: Vec<PathBuf>,
        authorize: impl Fn(&std::path::Path) -> AppResult<()>,
    ) -> AppResult<Self> {
        validate(attachments, authorize).map(Self)
    }

    pub(super) fn from_renderer<R: Runtime>(
        app: &AppHandle<R>,
        attachments: Vec<PathBuf>,
    ) -> AppResult<Self> {
        let scope = app.fs_scope();
        validate(attachments, |path| {
            if scope.is_allowed(path) {
                Ok(())
            } else {
                Err(AppError::invalid(
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
    authorize: impl Fn(&std::path::Path) -> AppResult<()>,
) -> AppResult<Vec<PathBuf>> {
    attachments
        .into_iter()
        .map(|source| {
            authorize(&source)?;
            let metadata = fs::symlink_metadata(&source).map_err(|error| {
                AppError::invalid(format!("Cannot inspect attachment: {error}"))
            })?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(AppError::invalid("Agent attachments must be regular files"));
            }
            let canonical = source
                .canonicalize()
                .map_err(|error| AppError::invalid(format!("Cannot open attachment: {error}")))?;
            authorize(&canonical)?;
            Ok(canonical)
        })
        .collect()
}
