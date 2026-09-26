use std::path::{Path, PathBuf};

use tokio_util::sync::CancellationToken;

use crate::spaces::assets::enrichment::ensure_active;

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
    let target = paths::preview_path(
        app,
        source,
        &format!("{}.system-thumbnail.png", uuid::Uuid::new_v4()),
    )?;
    if let Err(error) = crate::system_preview::thumbnail::save(source, &target, cancellation) {
        if target.is_file() {
            if let Err(cleanup) = std::fs::remove_file(&target) {
                log::warn!("failed thumbnail cleanup: {cleanup}");
            }
        }
        return Err(error);
    }
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
    let target = paths::preview_path(app, source, &format!("{}.shell.png", uuid::Uuid::new_v4()))?;
    if let Err(error) = crate::system_preview::thumbnail::save(source, &target, cancellation) {
        if target.is_file() {
            if let Err(cleanup) = std::fs::remove_file(&target) {
                log::warn!("failed thumbnail cleanup: {cleanup}");
            }
        }
        return Err(error);
    }
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
