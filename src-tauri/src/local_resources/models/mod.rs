use std::{fs, path::PathBuf};

use tauri::{AppHandle, Manager, Runtime};

pub mod catalog;
mod direct_download;
mod download;
pub(crate) mod installation;
pub(crate) mod model_archive;
pub(crate) mod snapshots;
pub(crate) mod sources;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
pub(crate) struct ModelDescriptor {
    pub snapshot: snapshots::SnapshotKind,
    pub file: &'static str,

    pub sha256: &'static str,
    pub download_size: u64,
}

#[derive(Clone, serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub installed: bool,
    pub verified: bool,
    pub path: String,
    pub size: Option<u64>,
    pub download_size: u64,
}

pub(crate) fn installation_path<R: Runtime>(
    app: &AppHandle<R>,
    descriptor: ModelDescriptor,
) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| error.to_string())
        .map(|root| {
            root.join("local-resources")
                .join("models")
                .join(descriptor.snapshot.name())
                .join(descriptor.snapshot.digest())
                .join(descriptor.file)
        })
}

pub(crate) fn status<R: Runtime>(
    app: &AppHandle<R>,
    descriptor: ModelDescriptor,
) -> Result<ModelStatus, String> {
    let path = installation_path(app, descriptor)?;
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
        Some(value) => super::verification::receipt_matches(&path, descriptor.sha256, value)?,
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

pub(crate) fn require_verified_path<R: Runtime>(
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
