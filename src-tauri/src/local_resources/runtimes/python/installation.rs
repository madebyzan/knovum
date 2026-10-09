use std::fs;

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::{
    archive,
    coordinator::update,
    descriptor::{self, Descriptor},
    verification, RuntimeError, MANIFEST_NAME,
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
    let installation_lease = loop {
        if destination.exists() {
            let Some(lease) = super::retention::try_acquire(&destination)? else {
                tokio::select! {
                    biased;
                    _ = cancellation.cancelled() => return Err(crate::local_resources::preparation::RESOURCE_PREPARATION_CANCELLED.into()),
                    () = tokio::time::sleep(std::time::Duration::from_millis(10)) => {}
                }
                continue;
            };
            if !destination.exists() {
                drop(lease);
                continue;
            }
            match verify_installed_async(&destination, descriptor, cancellation).await {
                Ok(()) => {
                    retain_installed(app, descriptor, &destination, lease).await?;
                    progress(update(
                        ResourcePreparationStage::Ready,
                        descriptor.size,
                        descriptor.size,
                    ));
                    return Ok(());
                }
                Err(error @ RuntimeError::InvalidPublication(_)) => {
                    drop(lease);
                    if let Some(installation) =
                        super::retention::try_acquire_exclusive(&destination)?
                    {
                        break installation;
                    }
                    return Err(format!(
                        "Installed runtime is active and cannot be replaced: {error}"
                    ));
                }
                Err(error) => return Err(error.into()),
            }
        }
        if let Some(installation) = super::retention::try_acquire_exclusive(&destination)? {
            break installation;
        }
        tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(crate::local_resources::preparation::RESOURCE_PREPARATION_CANCELLED.into()),
            () = tokio::time::sleep(std::time::Duration::from_millis(10)) => {}
        }
    };
    if destination.exists() {
        match verify_installed_async(&destination, descriptor, cancellation).await {
            Ok(()) => {
                let lease = share_installation(installation_lease).await?;
                retain_installed(app, descriptor, &destination, lease).await?;
                progress(update(
                    ResourcePreparationStage::Ready,
                    descriptor.size,
                    descriptor.size,
                ));
                return Ok(());
            }
            Err(RuntimeError::InvalidPublication(_)) => {
                let invalid = destination.clone();
                crate::blocking::run(move || filesystem::remove_directory(&invalid)).await?;
            }
            Err(error) => return Err(error.into()),
        }
    }
    let staging = Staging::create(parent, &descriptor.sha256)?;
    prepare_staging(
        descriptor,
        &descriptor.archive_url,
        &staging.0,
        cancellation,
        progress,
    )
    .await?;
    archive::require_active(cancellation)?;
    let prepared_runtime = staging.0.join("runtime");
    let published_destination = destination.clone();
    let lease = crate::blocking::run(move || {
        installation_lease.publish_directory(&prepared_runtime, &published_destination)
    })
    .await?;
    retain_installed(app, descriptor, &destination, lease).await?;
    progress(update(
        ResourcePreparationStage::Ready,
        descriptor.size,
        descriptor.size,
    ));
    Ok(())
}

fn verify_installed(
    destination: &std::path::Path,
    descriptor: &Descriptor,
    cancellation: &CancellationToken,
) -> Result<(), RuntimeError> {
    let manifest_path = destination.join(MANIFEST_NAME);
    let bytes = verification::read_manifest(&manifest_path)?;
    let digest = verification::digest(&bytes);
    if digest != descriptor.manifest_sha256 {
        return Err(RuntimeError::InvalidPublication(
            "installed manifest digest differs".into(),
        ));
    }
    verification::verify_runtime(
        &manifest_path,
        &bytes,
        digest,
        cancellation,
        Some(descriptor),
        descriptor.profile,
    )
    .map(|_| ())
}

async fn verify_installed_async(
    destination: &std::path::Path,
    descriptor: &Descriptor,
    cancellation: &CancellationToken,
) -> Result<(), RuntimeError> {
    let destination = destination.to_path_buf();
    let descriptor = descriptor.clone();
    let cancellation = cancellation.clone();
    crate::blocking::run(move || verify_installed(&destination, &descriptor, &cancellation)).await
}

async fn retain_installed<R: Runtime>(
    app: &AppHandle<R>,
    descriptor: &Descriptor,
    destination: &std::path::Path,
    lease: std::sync::Arc<super::RuntimeLease>,
) -> Result<(), String> {
    let handle = app.clone();
    let profile = descriptor.profile;
    let manifest = destination.join(MANIFEST_NAME);
    crate::blocking::run(move || {
        super::resolve_with_lease(&handle, profile, manifest, lease)?;
        super::retire_unused(&handle, profile)
    })
    .await
}

async fn share_installation(
    installation: super::retention::InstallationLease,
) -> Result<std::sync::Arc<super::RuntimeLease>, String> {
    crate::blocking::run(move || installation.into_shared()).await
}

struct Staging(std::path::PathBuf);

impl Staging {
    fn create(parent: &std::path::Path, identity: &str) -> Result<Self, String> {
        let path = parent.join(format!(".{identity}.{}.installing", uuid::Uuid::new_v4()));
        fs::create_dir(&path).map_err(|error| error.to_string())?;
        Ok(Self(path))
    }
}

impl Drop for Staging {
    fn drop(&mut self) {
        if let Err(error) = filesystem::remove_directory(&self.0) {
            log::error!("Python runtime installation staging cleanup failed: {error}");
        }
    }
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
        download::client()?.get(archive_url),
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
        return Err("Downloaded Python runtime archive digest differs".into());
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
            return Err(RuntimeError::InvalidPublication(
                "downloaded manifest digest differs".into(),
            ));
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
    .map_err(String::from)
}
