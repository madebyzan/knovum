use std::{fs, io, path::PathBuf};

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

    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(super) fn into_paths(self) -> Vec<PathBuf> {
        self.0
    }
}

pub(super) async fn stage(
    source: PathBuf,
    directory: &std::path::Path,
    index: usize,
) -> AiResult<PathBuf> {
    let current = source
        .canonicalize()
        .map_err(|error| AiError::invalid(format!("Cannot open attachment: {error}")))?;
    if current != source {
        return Err(AiError::invalid("Agent attachment changed before staging"));
    }
    let source_file = fs::File::open(&source)
        .map_err(|error| AiError::invalid(format!("Cannot open attachment: {error}")))?;
    verify_open_file(&source, &source_file)?;
    let name = source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| AiError::invalid("Attachment filename is not valid Unicode"))?;
    let destination = directory.join(format!("{index:03}-{name}"));
    let destination_for_copy = destination.clone();
    tokio::task::spawn_blocking(move || copy_open_file(source_file, &destination_for_copy))
        .await
        .map_err(|error| AiError::internal(format!("Attachment staging task failed: {error}")))?
        .map_err(|error| AiError::invalid(format!("Cannot stage attachment: {error}")))?;
    Ok(destination)
}

fn verify_open_file(source: &std::path::Path, source_file: &fs::File) -> AiResult<()> {
    let metadata = source_file
        .metadata()
        .map_err(|error| AiError::invalid(format!("Cannot inspect attachment: {error}")))?;
    if !metadata.is_file() {
        return Err(AiError::invalid("Agent attachment is not a regular file"));
    }
    let current = source
        .canonicalize()
        .map_err(|error| AiError::invalid(format!("Cannot open attachment: {error}")))?;
    let opened = same_file::Handle::from_file(
        source_file
            .try_clone()
            .map_err(|error| AiError::invalid(format!("Cannot verify attachment: {error}")))?,
    )
    .map_err(|error| AiError::invalid(format!("Cannot verify attachment: {error}")))?;
    let current_handle = same_file::Handle::from_path(&current)
        .map_err(|error| AiError::invalid(format!("Cannot verify attachment: {error}")))?;
    if current != source || opened != current_handle {
        return Err(AiError::invalid("Agent attachment changed before staging"));
    }
    Ok(())
}

fn copy_open_file(mut source: fs::File, destination: &std::path::Path) -> io::Result<()> {
    let mut destination = fs::File::create(destination)?;
    io::copy(&mut source, &mut destination)?;
    Ok(())
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
