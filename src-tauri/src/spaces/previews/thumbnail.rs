use std::path::{Path, PathBuf};

use tokio_util::sync::CancellationToken;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::spaces::assets::enrichment::ensure_active;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use super::paths;

#[cfg(target_os = "macos")]
pub(crate) fn generate(
    app: &tauri::AppHandle,
    source: &Path,
    cancellation: &CancellationToken,
) -> Result<Option<PathBuf>, String> {
    if !crate::system_preview::thumbnail::policy::supports(source) {
        return Ok(None);
    }
    ensure_active(cancellation)?;
    let target = paths::preview_path(app, source, "system-thumbnail.png")?;
    crate::system_preview::thumbnail::save(source, &target, cancellation)?;
    Ok(Some(target))
}

#[cfg(target_os = "windows")]
pub(crate) fn generate(
    app: &tauri::AppHandle,
    source: &Path,
    cancellation: &CancellationToken,
) -> Result<Option<PathBuf>, String> {
    if !crate::system_preview::thumbnail::policy::supports(source) {
        return Ok(None);
    }
    ensure_active(cancellation)?;
    let target = paths::preview_path(app, source, "shell.png")?;
    crate::system_preview::thumbnail::save(source, &target, cancellation)?;
    Ok(Some(target))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) fn generate(
    app: &tauri::AppHandle,
    source: &Path,
    cancellation: &CancellationToken,
) -> Result<Option<PathBuf>, String> {
    let _ = (app, source, cancellation);
    Ok(None)
}
