use std::fs;

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::{
    archive,
    coordinator::update,
    descriptor::{self, Descriptor},
    verification, MANIFEST_NAME,
};
use crate::local_resources::{
    download, filesystem,
    preparation::contracts::{ResourcePreparationProgress, ResourcePreparationStage},
};

pub(super) async fn install<R: Runtime>(
    app: &AppHandle<R>,
    descriptor: &Descriptor,
    cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<(), String> {
    let destination = descriptor::destination(app, descriptor)?;
    let parent = destination
        .parent()
        .ok_or("Runtime installation directory is unavailable")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let _lease = super::retention::acquire(&destination)?;
    let staging = parent.join(format!(".{}.installing", descriptor.sha256));
    filesystem::remove_directory(&staging)?;
    fs::create_dir(&staging).map_err(|error| error.to_string())?;
    let region = crate::local_resources::preferences::region(app)?;
    let prepared = prepare_staging(
        descriptor,
        descriptor.sources.url(region),
        &staging,
        cancellation,
        progress,
    )
    .await;
    if let Err(error) = prepared {
        filesystem::remove_directory(&staging)?;
        return Err(error);
    }
    if let Err(error) = archive::require_active(cancellation) {
        filesystem::remove_directory(&staging)?;
        return Err(error);
    }
    // A corrupt installation never becomes a usable runtime. Its identity directory is replaced only after verification.
    filesystem::remove_directory(&destination)?;
    fs::rename(staging.join("runtime"), &destination).map_err(|error| error.to_string())?;
    filesystem::remove_directory(&staging)?;
    super::retention::retire_unused(&destination)?;
    progress(update(
        ResourcePreparationStage::Ready,
        descriptor.size,
        descriptor.size,
    ));
    Ok(())
}

async fn prepare_staging(
    descriptor: &Descriptor,
    archive_url: &str,
    staging: &std::path::Path,
    cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<(), String> {
    let archive_path = staging.join("runtime.tar.gz");
    download::to_path(
        &download::client()?,
        reqwest::Url::parse(archive_url).map_err(|error| error.to_string())?,
        &archive_path,
        download::DownloadPolicy {
            maximum_bytes: descriptor.size,
            required_bytes: Some(descriptor.size),
            cancellation_message:
                crate::local_resources::preparation::RESOURCE_PREPARATION_CANCELLED,
        },
        cancellation,
        &mut |bytes, _| {
            progress(update(
                ResourcePreparationStage::Downloading,
                bytes,
                descriptor.size,
            ))
        },
    )
    .await
    .map_err(download::DownloadError::message)?;
    progress(update(
        ResourcePreparationStage::Verifying,
        descriptor.size,
        descriptor.size,
    ));
    if !filesystem::verify_file(
        archive_path.clone(),
        descriptor.sha256.clone(),
        cancellation.clone(),
    )
    .await?
    {
        return Err("python-runtime-invalid: downloaded archive digest differs".into());
    }
    let runtime = staging.join("runtime");
    fs::create_dir(&runtime).map_err(|error| error.to_string())?;
    progress(update(
        ResourcePreparationStage::Installing,
        descriptor.size,
        descriptor.size,
    ));
    archive::unpack(archive_path, runtime.clone(), cancellation.clone()).await?;
    progress(update(
        ResourcePreparationStage::Verifying,
        descriptor.size,
        descriptor.size,
    ));
    let manifest_path = runtime.join(MANIFEST_NAME);
    let expected = descriptor.manifest_sha256.clone();
    let verification_cancellation = cancellation.clone();
    let descriptor = descriptor.clone();
    tokio::task::spawn_blocking(move || {
        let bytes = verification::read_manifest(&manifest_path)?;
        let digest = verification::digest(&bytes);
        if digest != expected {
            return Err("python-runtime-invalid: downloaded manifest digest differs".into());
        }
        verification::verify_runtime(
            &manifest_path,
            &bytes,
            digest,
            &verification_cancellation,
            Some(&descriptor),
            descriptor.profile,
        )
        .map(|_| ())
    })
    .await
    .map_err(|error| error.to_string())?
}
