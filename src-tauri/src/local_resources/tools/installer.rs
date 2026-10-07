use std::{fs, path::Path};

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::{
    catalog::{MediaImportToolKind, ToolDescriptor},
    filesystem, ToolPreparationProgress,
};
use crate::local_resources::download;
use crate::local_resources::preparation::contracts::ResourcePreparationStage;
use crate::local_resources::preparation::RESOURCE_PREPARATION_CANCELLED;

pub(super) async fn install_tool<R: Runtime>(
    app: &AppHandle<R>,
    descriptor: &ToolDescriptor,
    cancellation: &CancellationToken,
    progress: &mut impl FnMut(ToolPreparationProgress),
) -> Result<(), String> {
    let destination = filesystem::installation_directory(app, descriptor)?;
    let installed_binary = destination.join(descriptor.binary_name);
    if installed_binary.is_file() {
        progress(ToolPreparationProgress {
            stage: ResourcePreparationStage::Verifying,
            downloaded: 0,
            total: fs::metadata(&installed_binary)
                .ok()
                .map(|metadata| metadata.len()),
        });
    }
    if installed_binary.is_file()
        && filesystem::verify_and_record_file(
            installed_binary,
            descriptor.binary_sha256.clone(),
            cancellation.clone(),
        )
        .await?
    {
        return Ok(());
    }
    let parent = destination
        .parent()
        .ok_or("Media import tool installation directory is unavailable")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let staging = parent.join(format!(".{}.installing", descriptor.version));
    filesystem::remove_directory(&staging)?;
    fs::create_dir(&staging).map_err(|error| error.to_string())?;
    if let Err(error) = install_to_staging(descriptor, &staging, cancellation, progress).await {
        if let Err(cleanup_error) = filesystem::remove_directory(&staging) {
            log::warn!("failed to clean media import tool staging: {cleanup_error}");
        }
        return Err(error);
    }
    if cancellation.is_cancelled() {
        filesystem::remove_directory(&staging)?;
        return Err(RESOURCE_PREPARATION_CANCELLED.to_string());
    }
    if destination.exists() {
        fs::remove_dir_all(&destination).map_err(|error| error.to_string())?;
    }
    fs::rename(&staging, &destination).map_err(|error| error.to_string())?;
    super::super::verification::record(
        &destination.join(descriptor.binary_name),
        &descriptor.binary_sha256,
    )
}

async fn install_to_staging(
    descriptor: &ToolDescriptor,
    staging: &Path,
    cancellation: &CancellationToken,
    progress: &mut impl FnMut(ToolPreparationProgress),
) -> Result<(), String> {
    let archive = staging.join(&descriptor.asset);
    let client = download::client()?;
    let url = reqwest::Url::parse(&descriptor.url).map_err(|error| error.to_string())?;
    let mut downloaded = 0_u64;
    let mut total = None;
    download::to_path(
        client.get(url),
        &archive,
        download::DownloadPolicy {
            maximum_bytes: 256 * 1024 * 1024,
            required_bytes: None,
            cancellation_message: RESOURCE_PREPARATION_CANCELLED,
        },
        cancellation,
        &mut |current, expected| {
            downloaded = current;
            total = expected;
            progress(ToolPreparationProgress {
                stage: ResourcePreparationStage::Downloading,
                downloaded: current,
                total: expected,
            });
        },
    )
    .await
    .map_err(download::DownloadError::message)?;
    progress(ToolPreparationProgress {
        stage: ResourcePreparationStage::Verifying,
        downloaded,
        total,
    });
    if !filesystem::verify_file(
        archive.clone(),
        descriptor.archive_sha256.clone(),
        cancellation.clone(),
    )
    .await?
    {
        return Err(format!(
            "Downloaded {} failed SHA-256 verification",
            descriptor.binary_name
        ));
    }
    let binary = staging.join(descriptor.binary_name);
    progress(ToolPreparationProgress {
        stage: ResourcePreparationStage::Installing,
        downloaded,
        total,
    });
    match descriptor.kind {
        MediaImportToolKind::YtDlp => {
            if archive != binary {
                fs::rename(&archive, &binary).map_err(|error| error.to_string())?;
            }
        }
        MediaImportToolKind::Deno => {
            super::deno_archive::extract(
                archive.clone(),
                binary.clone(),
                descriptor.binary_name.to_string(),
                cancellation.clone(),
            )
            .await?;
            fs::remove_file(&archive).map_err(|error| error.to_string())?;
        }
    }
    progress(ToolPreparationProgress {
        stage: ResourcePreparationStage::Verifying,
        downloaded,
        total,
    });
    if !filesystem::finalize_executable(
        binary.clone(),
        descriptor.binary_sha256.clone(),
        cancellation.clone(),
    )
    .await?
    {
        return Err(format!(
            "Extracted {} failed SHA-256 verification",
            descriptor.binary_name
        ));
    }
    Ok(())
}
