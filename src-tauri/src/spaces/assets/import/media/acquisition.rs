use std::{fs, path::Path};
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::{contracts::ImportedMedia, publish};
use crate::media::acquisition::{
    contracts::{AcquisitionProgress, MediaKind},
    metadata::is_http_url,
    tools::AcquisitionTools,
    yt_dlp::{self, AcquisitionError},
};
use crate::spaces::assets::asset_directory;

pub(super) fn run(
    app: &tauri::AppHandle,
    job_id: Uuid,
    kind: MediaKind,
    source_url: String,
    tools: AcquisitionTools,
    progress: &Channel<AcquisitionProgress>,
    cancellation: &CancellationToken,
) -> Result<Option<ImportedMedia>, String> {
    let assets = asset_directory(app)?;
    let workspace = assets.join(format!(".media-import-{job_id}"));
    fs::create_dir(&workspace)
        .map_err(|error| format!("Unable to create media import workspace: {error}"))?;
    let acquired_media = execute(
        &assets,
        &workspace,
        job_id,
        kind,
        source_url,
        tools,
        progress,
        cancellation,
    );
    if let Err(error) = fs::remove_dir_all(&workspace) {
        log::warn!(
            "failed to remove media import workspace {}: {error}",
            workspace.display()
        );
    }
    acquired_media
}

fn execute(
    assets: &Path,
    workspace: &Path,
    job_id: Uuid,
    kind: MediaKind,
    source_url: String,
    tools: AcquisitionTools,
    progress: &Channel<AcquisitionProgress>,
    cancellation: &CancellationToken,
) -> Result<Option<ImportedMedia>, String> {
    match yt_dlp::acquire(
        &tools.yt_dlp,
        &tools.deno,
        &tools.ffmpeg,
        &source_url,
        kind,
        workspace,
        progress,
        cancellation,
    ) {
        Ok(()) => {}
        Err(AcquisitionError::Unsupported) => return Ok(None),
        Err(AcquisitionError::Failed(error)) => return Err(error),
    }
    progress
        .send(AcquisitionProgress {
            stage: "finalizing",
            downloaded_bytes: 0,
            total_bytes: None,
            processed_ms: None,
            duration_ms: None,
        })
        .map_err(|error| error.to_string())?;
    publish::commit(
        assets,
        workspace,
        job_id,
        kind,
        &tools.ffmpeg,
        progress,
        cancellation,
    )
    .map(Some)
}

pub(super) fn parse_job_id(value: &str) -> Result<Uuid, String> {
    let id = Uuid::parse_str(value).map_err(|_| "Media import job ID must be a UUID")?;
    if id.to_string() != value {
        return Err("Media import job ID must use canonical UUID form".into());
    }
    Ok(id)
}

pub(super) fn validate_source_url(value: &str) -> Result<(), String> {
    if value.len() > 4096 || !is_http_url(value) {
        Err("Media source URL is invalid".into())
    } else {
        Ok(())
    }
}

pub(super) fn cleanup_stale(app: &tauri::AppHandle) -> Result<(), String> {
    let assets = asset_directory(app)?;
    for entry in fs::read_dir(assets).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let name = entry.file_name();
        let Some(id) = name
            .to_str()
            .and_then(|name| name.strip_prefix(".media-import-"))
        else {
            continue;
        };
        if Uuid::parse_str(id).is_ok() && entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            fs::remove_dir_all(entry.path()).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}
