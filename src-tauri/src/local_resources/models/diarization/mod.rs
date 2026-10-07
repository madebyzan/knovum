use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime};

pub mod commands;
mod filesystem;
mod installation;
mod manifest;
pub(crate) use installation::prepare;
pub(crate) const ACCESS_TOKEN_KEY: &str = "local-model:community-1:access-token";
pub(crate) const RUNTIME_PROFILE: crate::local_resources::runtimes::python::RuntimeProfile =
    crate::local_resources::runtimes::python::RuntimeProfile::TorchInference;
pub const REVISION: &str = manifest::REVISION;

use filesystem::{remove_owned_directory, verify_directory, verify_managed_root};

#[derive(Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DiarizationModelStatus {
    pub installed: bool,
    pub verified: bool,
    pub path: String,
    pub revision: String,
    pub total_bytes: u64,
    pub token_configured: bool,
}

pub struct VerifiedDiarizationDirectory(PathBuf);

impl VerifiedDiarizationDirectory {
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

pub fn installation_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| error.to_string())
        .map(|root| {
            root.join("local-resources")
                .join("models")
                .join("pyannote-community-1-knovum")
        })
}

pub fn status<R: Runtime>(
    app: &AppHandle<R>,
    token_configured: bool,
) -> Result<DiarizationModelStatus, String> {
    let path = installation_path(app)?;
    Ok(DiarizationModelStatus {
        installed: path.is_dir(),
        verified: path.is_dir() && verify_installation(app, &path).is_ok(),
        path: path.to_string_lossy().into_owned(),
        revision: manifest::REVISION.into(),
        total_bytes: manifest::TOTAL_BYTES,
        token_configured,
    })
}

pub fn require_verified_directory<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<VerifiedDiarizationDirectory, String> {
    let path = installation_path(app)?;
    if !path.is_dir() {
        return Err("Diarization model is not installed".into());
    }
    verify_installation(app, &path)
        .map_err(|error| format!("Diarization model installation is corrupt: {error}"))?;
    Ok(VerifiedDiarizationDirectory(path))
}

pub(super) fn remove<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let target = installation_path(app)?;
    match fs::symlink_metadata(&target) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
        Ok(_) => {}
    }
    let parent = target
        .parent()
        .ok_or("Diarization model directory is unavailable")?;
    verify_managed_root(app, parent)?;
    remove_owned_directory(&target, parent)
}

fn verify_installation<R: Runtime>(app: &AppHandle<R>, path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or("Diarization model directory is unavailable")?;
    verify_managed_root(app, parent)?;
    verify_directory(path)
}

#[cfg(test)]
mod tests;
