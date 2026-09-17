use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime};
use tokio_util::sync::CancellationToken;

pub mod catalog;
pub(crate) mod commands;
mod download;
mod sources;
#[cfg(test)]
mod tests;
mod verification;

static INSTALL_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

#[derive(Clone, Copy)]
pub struct ModelDescriptor {
    pub directory: &'static str,
    pub file: &'static str,
    pub env_key: &'static str,
    pub repository: &'static str,
    pub revision: &'static str,
    pub sha256: &'static str,
    pub download_size: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub installed: bool,
    pub verified: bool,
    pub path: String,
    pub size: Option<u64>,
    pub download_size: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadProgress {
    pub downloaded: u64,
    pub total: Option<u64>,
    pub percent: Option<u8>,
}

pub fn resolve_model_path<R: Runtime>(
    app: &AppHandle<R>,
    descriptor: ModelDescriptor,
) -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os(descriptor.env_key)
        .map(PathBuf::from)
        .filter(|path| path.is_file())
    {
        return Ok(path);
    }
    installation_path(app, descriptor)
}

pub fn installation_path<R: Runtime>(
    app: &AppHandle<R>,
    descriptor: ModelDescriptor,
) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| error.to_string())
        .map(|root| {
            root.join("local-resources")
                .join("models")
                .join(descriptor.directory)
                .join(descriptor.file)
        })
}

pub fn status<R: Runtime>(
    app: &AppHandle<R>,
    descriptor: ModelDescriptor,
) -> Result<ModelStatus, String> {
    let path = resolve_model_path(app, descriptor)?;
    let metadata = match fs::metadata(&path) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(format!(
                "Unable to inspect model {}: {error}",
                path.display()
            ));
        }
    };
    let verified = match metadata.as_ref() {
        Some(value) => verification::receipt_matches(&path, descriptor, value)?,
        None => false,
    };
    Ok(ModelStatus {
        installed: metadata.is_some(),
        verified,
        path: path.to_string_lossy().into_owned(),
        size: metadata.map(|value| value.len()),
        download_size: descriptor.download_size,
    })
}

pub fn require_verified_path<R: Runtime>(
    app: &AppHandle<R>,
    descriptor: ModelDescriptor,
) -> Result<PathBuf, String> {
    let model_status = status(app, descriptor)?;
    if !model_status.verified {
        return Err(format!(
            "Required model {} is not installed or failed verification",
            descriptor.file
        ));
    }
    Ok(PathBuf::from(model_status.path))
}

pub async fn install<R: Runtime>(
    app: &AppHandle<R>,
    descriptor: ModelDescriptor,
    progress: &mut impl FnMut(ModelDownloadProgress) -> Result<(), String>,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let _guard = INSTALL_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await;
    let target = installation_path(app, descriptor)?;
    if target.is_file() && verification::verify_and_record(&target, descriptor)? {
        let size = fs::metadata(&target)
            .map_err(|error| error.to_string())?
            .len();
        return progress(download::download_progress(size, Some(size)));
    }
    let parent = target.parent().ok_or("Model directory is unavailable")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = target.with_extension("download");
    if temporary.exists() {
        fs::remove_file(&temporary).map_err(|error| error.to_string())?;
    }
    if let Err(error) = download::to_path(descriptor, &temporary, progress, cancellation).await {
        remove_temporary(&temporary);
        return Err(error);
    }
    if !verification::verify(&temporary, descriptor)? {
        remove_temporary(&temporary);
        return Err(format!(
            "Downloaded model {} failed SHA-256 verification",
            descriptor.file
        ));
    }
    if target.exists() {
        fs::remove_file(&target).map_err(|error| error.to_string())?;
    }
    verification::remove_receipt(&target)?;
    fs::rename(&temporary, &target).map_err(|error| error.to_string())?;
    verification::record(&target, descriptor)
}

fn remove_temporary(path: &Path) {
    if let Err(error) = fs::remove_file(path) {
        if error.kind() != std::io::ErrorKind::NotFound {
            log::warn!(
                "failed to remove temporary model file {}: {error}",
                path.display()
            );
        }
    }
}

pub fn uninstall<R: Runtime>(
    app: &AppHandle<R>,
    descriptor: ModelDescriptor,
) -> Result<(), String> {
    let path = installation_path(app, descriptor)?;
    if path.exists() {
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }
    verification::remove_receipt(&path)?;
    Ok(())
}
