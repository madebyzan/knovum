use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager, Runtime};
use tokio_util::sync::CancellationToken;

pub(crate) fn remove_directory(path: &Path) -> Result<(), String> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

pub(crate) async fn verify_file(
    path: PathBuf,
    expected_sha256: String,
    cancellation: CancellationToken,
) -> Result<bool, String> {
    crate::foundation::blocking::run(move || {
        super::verification::verify_cancellable(
            &path,
            &expected_sha256,
            &cancellation,
            super::preparation::RESOURCE_PREPARATION_CANCELLED,
        )
    })
    .await
}

pub(crate) fn verify_regular_directory(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        return Err(format!("Model directory is linked: {}", path.display()));
    }
    Ok(())
}

/// Installed bytes of a verified snapshot, which bound the load that reads it.
///
/// Metadata only: these bytes were hashed when the snapshot was installed, and the caller needs
/// their size rather than their content.
pub(crate) fn installed_bytes(path: &Path) -> Result<u64, String> {
    let mut total: u64 = 0;
    for entry in walkdir::WalkDir::new(path).follow_links(false) {
        let metadata = entry
            .map_err(|error| error.to_string())?
            .metadata()
            .map_err(|error| error.to_string())?;
        if metadata.is_file() {
            total = total.saturating_add(metadata.len());
        }
    }
    Ok(total)
}

pub(crate) fn verify_managed_root<R: Runtime>(
    app: &AppHandle<R>,
    root: &Path,
) -> Result<(), String> {
    verify_regular_directory(root)?;
    let local_resources = root
        .parent()
        .ok_or("Local resource directory is unavailable")?;
    verify_regular_directory(local_resources)?;
    let app_data_directory = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    let canonical_app_data_directory = app_data_directory
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let canonical_local_resources = local_resources
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let canonical_root = root.canonicalize().map_err(|error| error.to_string())?;
    if !canonical_root.starts_with(&canonical_app_data_directory)
        || canonical_root.parent() != Some(canonical_local_resources.as_path())
    {
        return Err("Model directory escapes application data".into());
    }
    Ok(())
}

pub(crate) fn remove_owned_directory(path: &Path, parent: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
        Ok(metadata) => {
            if path.parent() != Some(parent) {
                return Err("Model staging directory is outside its managed root".into());
            }
            if metadata.file_type().is_symlink() {
                return remove_link(path);
            }
            if is_reparse_point(&metadata) {
                return if metadata.is_dir() {
                    fs::remove_dir(path).map_err(|error| error.to_string())
                } else {
                    fs::remove_file(path).map_err(|error| error.to_string())
                };
            }
            if !metadata.is_dir() {
                return Err("Model managed path is not a directory".into());
            }
            remove_directory(path)
        }
    }
}

#[cfg(target_os = "windows")]
fn remove_link(path: &Path) -> Result<(), String> {
    fs::remove_dir(path)
        .or_else(|_| fs::remove_file(path))
        .map_err(|error| error.to_string())
}

#[cfg(not(target_os = "windows"))]
fn remove_link(path: &Path) -> Result<(), String> {
    fs::remove_file(path).map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}
#[cfg(not(target_os = "windows"))]
const fn is_reparse_point(_: &fs::Metadata) -> bool {
    false
}

#[cfg(test)]
#[path = "tests/filesystem.rs"]
mod tests;
