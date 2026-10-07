use std::{
    fs,
    path::Path,
    sync::{Arc, OnceLock},
};

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::{filesystem, manifest};
use crate::local_resources::{
    download,
    preparation::{
        contracts::{ResourcePreparationProgress, ResourcePreparationStage},
        locks,
        shared_installation::Registry,
        RESOURCE_PREPARATION_CANCELLED,
    },
};

pub(crate) async fn prepare<R: Runtime>(
    app: &AppHandle<R>,
    cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<(), String> {
    static INSTALLATIONS: OnceLock<Arc<Registry<ResourcePreparationProgress>>> = OnceLock::new();
    let handle = app.clone();
    let mut waiter = INSTALLATIONS
        .get_or_init(Registry::new)
        .join(
            cancellation,
            update(ResourcePreparationStage::Queued, 0),
            move |installation| async move {
                install(&handle, &installation.cancellation, &mut |value| {
                    installation.publish(value)
                })
                .await
            },
        )
        .await?;
    waiter.wait(cancellation, progress).await?;
    progress(update(
        ResourcePreparationStage::Ready,
        manifest::TOTAL_BYTES,
    ));
    Ok(())
}

async fn install<R: Runtime>(
    app: &AppHandle<R>,
    cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<(), String> {
    let _installation = tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Err(RESOURCE_PREPARATION_CANCELLED.into()),
        guard = locks::acquire_diarization_installation() => guard,
    };
    let target = super::installation_path(app)?;
    if target.is_dir() {
        let handle = app.clone();
        let checked = target.clone();
        crate::blocking::run(move || super::verify_installation(&handle, &checked))
            .await
            .map_err(|error| {
                format!("Diarization model is corrupt; remove it before downloading again: {error}")
            })?;
        return Ok(());
    }
    let token = crate::secrets::read_secret(super::ACCESS_TOKEN_KEY).await?
        .ok_or("Authorize Community-1 on Hugging Face and save a read access token in AI model settings")?;
    let parent = target
        .parent()
        .ok_or("Diarization model directory is unavailable")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    filesystem::verify_managed_root(app, parent)?;
    let staging = parent.join(".pyannote-community-1-knovum.downloading");
    filesystem::remove_owned_directory(&staging, parent)?;
    fs::create_dir(&staging).map_err(|error| error.to_string())?;
    let outcome = download_files(&staging, &token, cancellation, progress).await;
    if let Err(error) = outcome {
        filesystem::remove_owned_directory(&staging, parent)?;
        return Err(error);
    }
    if cancellation.is_cancelled() {
        filesystem::remove_owned_directory(&staging, parent)?;
        return Err(RESOURCE_PREPARATION_CANCELLED.into());
    }
    fs::rename(&staging, &target).map_err(|error| error.to_string())
}

async fn download_files(
    staging: &Path,
    token: &str,
    cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<(), String> {
    let client = download::client()?;
    let mut completed = 0;
    for file in manifest::FILES {
        let destination = staging.join(file.path);
        fs::create_dir_all(
            destination
                .parent()
                .ok_or("Diarization file directory is unavailable")?,
        )
        .map_err(|error| error.to_string())?;
        let url = format!(
            "https://huggingface.co/pyannote/speaker-diarization-community-1/resolve/{}/{}",
            manifest::REVISION,
            file.path
        );
        download::to_path(client.get(url).bearer_auth(token), &destination, download::DownloadPolicy {
            maximum_bytes: file.size, required_bytes: Some(file.size),
            cancellation_message: RESOURCE_PREPARATION_CANCELLED,
        }, cancellation, &mut |bytes, _| progress(update(ResourcePreparationStage::Downloading, completed + bytes)))
            .await.map_err(|error| match error {
                download::DownloadError::Http { status: 401 | 403, .. } =>
                    "Hugging Face denied Community-1 access. Authorize this model and use a token with read access to gated repositories".into(),
                error => error.message(),
            })?;
        completed += file.size;
    }
    fs::write(staging.join("config.yaml"), manifest::CONFIG).map_err(|error| error.to_string())?;
    progress(update(ResourcePreparationStage::Verifying, completed));
    let checked = staging.to_path_buf();
    crate::blocking::run(move || filesystem::verify_directory(&checked)).await
}

fn update(stage: ResourcePreparationStage, downloaded: u64) -> ResourcePreparationProgress {
    ResourcePreparationProgress {
        stage,
        downloaded,
        total: Some(manifest::TOTAL_BYTES),
        percent: Some(((downloaded * 100 / manifest::TOTAL_BYTES).min(100)) as u8),
    }
}
