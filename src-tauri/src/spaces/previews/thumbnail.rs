use std::path::{Path, PathBuf};

use tokio_util::sync::CancellationToken;

use crate::spaces::assets::enrichment::ensure_active;

use super::paths;

#[cfg(target_os = "macos")]
pub(crate) fn reserve(app: &tauri::AppHandle, source: &Path) -> Result<Option<PathBuf>, String> {
    if !crate::system_preview::thumbnail::policy::supports(source) {
        return Ok(None);
    }
    paths::preview_path(
        app,
        source,
        &format!("{}.system-thumbnail.png", uuid::Uuid::new_v4()),
    )
    .map(Some)
}

#[cfg(target_os = "windows")]
pub(crate) fn reserve(app: &tauri::AppHandle, source: &Path) -> Result<Option<PathBuf>, String> {
    if !crate::system_preview::thumbnail::policy::supports(source) {
        return Ok(None);
    }
    paths::preview_path(app, source, &format!("{}.shell.png", uuid::Uuid::new_v4())).map(Some)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) fn reserve(app: &tauri::AppHandle, source: &Path) -> Result<Option<PathBuf>, String> {
    let _ = (app, source);
    Ok(None)
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) fn generate_at(
    source: &Path,
    target: &Path,
    cancellation: &CancellationToken,
    claim_path: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<PathBuf, String> {
    ensure_active(cancellation)?;
    if let Err(error) =
        crate::system_preview::thumbnail::save_owned(source, target, cancellation, claim_path)
    {
        if target.is_file() {
            if let Err(cleanup) = std::fs::remove_file(target) {
                log::warn!("failed thumbnail cleanup: {cleanup}");
            }
        }
        return Err(error);
    }
    Ok(target.to_path_buf())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) fn generate_at(
    source: &Path,
    target: &Path,
    cancellation: &CancellationToken,
    claim_path: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<PathBuf, String> {
    let _ = (source, target, cancellation, claim_path);
    Err("System thumbnails are unavailable on this platform".into())
}

#[cfg(target_os = "macos")]
pub(crate) fn generate(
    app: &tauri::AppHandle,
    source: &Path,
    cancellation: &CancellationToken,
) -> Result<Option<PathBuf>, String> {
    let Some(target) = reserve(app, source)? else {
        return Ok(None);
    };
    generate_at(source, &target, cancellation, &mut |_| Ok(())).map(Some)
}

#[cfg(target_os = "windows")]
pub(crate) fn generate(
    app: &tauri::AppHandle,
    source: &Path,
    cancellation: &CancellationToken,
) -> Result<Option<PathBuf>, String> {
    let Some(target) = reserve(app, source)? else {
        return Ok(None);
    };
    generate_at(source, &target, cancellation, &mut |_| Ok(())).map(Some)
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
