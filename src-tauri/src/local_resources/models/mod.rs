use std::{
    fs,
    path::{Path, PathBuf},
};

use tauri::{AppHandle, Manager, Runtime};
use tokio_util::sync::CancellationToken;

use super::preparation::contracts::{ResourcePreparationProgress, ResourcePreparationStage};

pub mod catalog;
mod download;
mod sources;
#[cfg(test)]
mod tests;

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

#[derive(Clone, serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub installed: bool,
    pub verified: bool,
    pub path: String,
    pub size: Option<u64>,
    pub download_size: u64,
}

pub fn resolve_model_path<R: Runtime>(
    app: &AppHandle<R>,
    descriptor: ModelDescriptor,
) -> Result<PathBuf, String> {
    if let Some(path) = super::configured_path::configured_path(descriptor.env_key)? {
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

pub(super) async fn install<R: Runtime>(
    app: &AppHandle<R>,
    descriptor: ModelDescriptor,
    progress: &mut impl FnMut(ResourcePreparationProgress),
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let target = installation_path(app, descriptor)?;
    if target.is_file() {
        progress(ResourcePreparationProgress {
            stage: ResourcePreparationStage::Verifying,
            downloaded: 0,
            total: Some(descriptor.download_size),
            percent: Some(0),
        });
    }
    let target_for_verification = target.clone();
    let verification_cancellation = cancellation.clone();
    let already_verified = target.is_file()
        && crate::blocking::run(move || {
            super::verification::verify_and_record_cancellable(
                &target_for_verification,
                descriptor.sha256,
                &verification_cancellation,
                "Model verification cancelled",
            )
        })
        .await?;
    if already_verified {
        let size = fs::metadata(&target)
            .map_err(|error| error.to_string())?
            .len();
        progress(ResourcePreparationProgress {
            stage: ResourcePreparationStage::Ready,
            downloaded: size,
            total: Some(size),
            percent: Some(100),
        });
        return Ok(());
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
    progress(ResourcePreparationProgress {
        stage: ResourcePreparationStage::Verifying,
        downloaded: descriptor.download_size,
        total: Some(descriptor.download_size),
        percent: Some(100),
    });
    let temporary_for_verification = temporary.clone();
    let verification_cancellation = cancellation.clone();
    let verified = match crate::blocking::run(move || {
        super::verification::verify_cancellable(
            &temporary_for_verification,
            descriptor.sha256,
            &verification_cancellation,
            "Model verification cancelled",
        )
    })
    .await
    {
        Ok(verified) => verified,
        Err(error) => {
            remove_temporary(&temporary);
            return Err(error);
        }
    };
    if !verified {
        remove_temporary(&temporary);
        return Err(format!(
            "Downloaded model {} failed SHA-256 verification",
            descriptor.file
        ));
    }
    if cancellation.is_cancelled() {
        remove_temporary(&temporary);
        return Err("Model installation cancelled".to_string());
    }
    if target.exists() {
        fs::remove_file(&target).map_err(|error| error.to_string())?;
    }
    super::verification::remove_receipt(&target)?;
    fs::rename(&temporary, &target).map_err(|error| error.to_string())?;
    super::verification::record(&target, descriptor.sha256)?;
    progress(ResourcePreparationProgress {
        stage: ResourcePreparationStage::Ready,
        downloaded: descriptor.download_size,
        total: Some(descriptor.download_size),
        percent: Some(100),
    });
    Ok(())
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
