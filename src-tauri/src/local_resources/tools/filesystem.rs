use std::{
    fs,
    path::{Path, PathBuf},
};

use tauri::{AppHandle, Manager, Runtime};
use tokio_util::sync::CancellationToken;

use super::catalog::ToolDescriptor;

pub(super) fn verified_installation_path<R: Runtime>(
    app: &AppHandle<R>,
    descriptor: &ToolDescriptor,
) -> Result<Option<PathBuf>, String> {
    let path = installation_path(app, descriptor)?;
    if !path.is_file() {
        return Ok(None);
    }
    let metadata = fs::metadata(&path).map_err(|error| error.to_string())?;
    Ok(
        super::super::verification::receipt_matches(&path, &descriptor.binary_sha256, &metadata)?
            .then_some(path),
    )
}

pub(super) fn installation_path<R: Runtime>(
    app: &AppHandle<R>,
    descriptor: &ToolDescriptor,
) -> Result<PathBuf, String> {
    Ok(installation_directory(app, descriptor)?.join(descriptor.binary_name))
}

pub(super) fn installation_directory<R: Runtime>(
    app: &AppHandle<R>,
    descriptor: &ToolDescriptor,
) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| error.to_string())
        .map(|root| {
            root.join("local-resources")
                .join("tools")
                .join(descriptor.directory)
                .join(&descriptor.version)
        })
}

pub(super) fn remove_directory(path: &Path) -> Result<(), String> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

pub(super) async fn verify_file(
    path: PathBuf,
    expected_sha256: String,
    cancellation: CancellationToken,
) -> Result<bool, String> {
    crate::blocking::run(move || {
        super::super::verification::verify_cancellable(
            &path,
            &expected_sha256,
            &cancellation,
            super::super::preparation::RESOURCE_PREPARATION_CANCELLED,
        )
    })
    .await
}

pub(super) async fn verify_and_record_file(
    path: PathBuf,
    expected_sha256: String,
    cancellation: CancellationToken,
) -> Result<bool, String> {
    crate::blocking::run(move || {
        super::super::verification::verify_and_record_cancellable(
            &path,
            &expected_sha256,
            &cancellation,
            super::super::preparation::RESOURCE_PREPARATION_CANCELLED,
        )
    })
    .await
}

pub(super) async fn finalize_executable(
    path: PathBuf,
    expected_sha256: String,
    cancellation: CancellationToken,
) -> Result<bool, String> {
    if !verify_file(path.clone(), expected_sha256, cancellation).await? {
        return Ok(false);
    }
    crate::blocking::run(move || {
        make_executable(&path)?;
        fs::File::open(path)
            .and_then(|file| file.sync_all())
            .map_err(|error| error.to_string())
    })
    .await?;
    Ok(true)
}

#[cfg(unix)]
pub(super) fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).map_err(|error| error.to_string())
}

#[cfg(not(unix))]
pub(super) fn make_executable(_: &Path) -> Result<(), String> {
    Ok(())
}
