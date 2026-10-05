use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_fs::FsExt;

pub mod commands;
mod filesystem;
mod manifest;
pub const REVISION: &str = manifest::REVISION;

use filesystem::{
    copy_verified, is_reparse_point, remove_owned_directory, verify_directory, verify_managed_root,
};

#[derive(Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DiarizationModelStatus {
    pub installed: bool,
    pub verified: bool,
    pub path: String,
    pub revision: String,
    pub total_bytes: u64,
    pub requires_user_terms: bool,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DiarizationModelImportReceipt {
    pub revision: String,
    pub installed_files: u8,
    pub total_bytes: u64,
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

pub fn status<R: Runtime>(app: &AppHandle<R>) -> Result<DiarizationModelStatus, String> {
    let path = installation_path(app)?;
    Ok(DiarizationModelStatus {
        installed: path.is_dir(),
        verified: path.is_dir() && verify_installation(app, &path).is_ok(),
        path: path.to_string_lossy().into_owned(),
        revision: manifest::REVISION.into(),
        total_bytes: manifest::TOTAL_BYTES,
        requires_user_terms: true,
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

pub(super) fn import<R: Runtime>(
    app: &AppHandle<R>,
    source_path: &str,
) -> Result<DiarizationModelImportReceipt, String> {
    let requested_source = PathBuf::from(source_path);
    let requested_metadata =
        fs::symlink_metadata(&requested_source).map_err(|error| error.to_string())?;
    if !requested_metadata.is_dir()
        || requested_metadata.file_type().is_symlink()
        || is_reparse_point(&requested_metadata)
    {
        return Err("Diarization import source must be a regular directory".into());
    }
    let source = requested_source
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if !app.fs_scope().is_allowed(&source) {
        return Err("Diarization import directory was not authorized by a user dialog".into());
    }
    let metadata = fs::symlink_metadata(&source).map_err(|error| error.to_string())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        return Err("Diarization import source must be a regular directory".into());
    }
    let target = installation_path(app)?;
    let parent = target
        .parent()
        .ok_or("Diarization model directory is unavailable")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    verify_managed_root(app, parent)?;
    match fs::symlink_metadata(&target) {
        Ok(_) if verify_installation(app, &target).is_ok() => {
            return Err("Diarization model is already installed".into());
        }
        Ok(_) => {
            return Err(
                "Diarization model installation is corrupt; remove it before importing again"
                    .into(),
            );
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.to_string()),
    }
    let staging = parent.join(".pyannote-community-1-knovum.importing");
    remove_owned_directory(&staging, parent)?;
    fs::create_dir(&staging).map_err(|error| error.to_string())?;
    let outcome = (|| {
        for file in manifest::FILES {
            copy_verified(&source.join(file.path), &staging.join(file.path), file)?;
        }
        fs::write(staging.join("config.yaml"), manifest::CONFIG)
            .map_err(|error| error.to_string())?;
        verify_directory(&staging)?;
        fs::rename(&staging, &target).map_err(|error| error.to_string())?;
        Ok(DiarizationModelImportReceipt {
            revision: manifest::REVISION.into(),
            installed_files: 4,
            total_bytes: manifest::TOTAL_BYTES,
        })
    })();
    if outcome.is_err() {
        let _ = remove_owned_directory(&staging, parent);
    }
    outcome
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
