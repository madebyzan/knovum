use std::sync::{Arc, OnceLock};

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use crate::local_resources::preparation::{
    contracts::{ResourcePreparationProgress, ResourcePreparationStage},
    shared_installation::Registry,
    RESOURCE_PREPARATION_CANCELLED,
};

pub(crate) async fn prepare<R: Runtime>(
    app: &AppHandle<R>,
    profile: super::RuntimeProfile,
    cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<(), String> {
    if cancellation.is_cancelled() {
        return Err(RESOURCE_PREPARATION_CANCELLED.into());
    }
    let handle = app.clone();
    if crate::blocking::run(move || {
        let runtime = super::resolve(&handle, profile)?;
        if runtime.is_some() {
            super::retire_unused(&handle, profile)?;
        }
        Ok::<_, String>(runtime)
    })
    .await?
    .is_some()
    {
        super::archive::require_active(cancellation)?;
        progress(update(ResourcePreparationStage::Ready, 0, 0));
        return Ok(());
    }
    let descriptor = super::descriptor::required(profile)?;
    static RESOURCES: OnceLock<Arc<Registry<ResourcePreparationProgress>>> = OnceLock::new();
    static LIGHT: OnceLock<Arc<Registry<ResourcePreparationProgress>>> = OnceLock::new();
    static TORCH: OnceLock<Arc<Registry<ResourcePreparationProgress>>> = OnceLock::new();
    static VOICE: OnceLock<Arc<Registry<ResourcePreparationProgress>>> = OnceLock::new();
    static AGENT: OnceLock<Arc<Registry<ResourcePreparationProgress>>> = OnceLock::new();
    let registry = match profile {
        super::RuntimeProfile::Resources => &RESOURCES,
        super::RuntimeProfile::LightInference => &LIGHT,
        super::RuntimeProfile::TorchInference => &TORCH,
        super::RuntimeProfile::VoiceGeneration => &VOICE,
        super::RuntimeProfile::Agent => &AGENT,
    }
    .get_or_init(Registry::new);
    let handle = app.clone();
    let descriptor_size = descriptor.size;
    let mut waiter = registry
        .join(
            cancellation,
            update(ResourcePreparationStage::Queued, 0, descriptor.size),
            move |installation| async move {
                super::installation::install(
                    &handle,
                    &descriptor,
                    &installation.cancellation,
                    &mut |progress| installation.publish(progress),
                )
                .await
            },
        )
        .await?;
    waiter.wait(cancellation, progress).await?;
    super::archive::require_active(cancellation)?;
    progress(update(
        ResourcePreparationStage::Ready,
        descriptor_size,
        descriptor_size,
    ));
    Ok(())
}

pub(super) fn update(
    stage: ResourcePreparationStage,
    downloaded: u64,
    total: u64,
) -> ResourcePreparationProgress {
    ResourcePreparationProgress {
        stage: match stage {
            ResourcePreparationStage::Downloading => ResourcePreparationStage::RuntimeDownloading,
            ResourcePreparationStage::Verifying => ResourcePreparationStage::RuntimeVerifying,
            ResourcePreparationStage::Installing | ResourcePreparationStage::Queued => {
                ResourcePreparationStage::RuntimeInstalling
            }
            value => value,
        },
        downloaded,
        total: Some(total),
        percent: Some(((downloaded.saturating_mul(100) / total.max(1)).min(100)) as u8),
    }
}
