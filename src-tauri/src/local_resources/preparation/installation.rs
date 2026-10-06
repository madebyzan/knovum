use std::sync::Arc;

use tauri::{AppHandle, Runtime};

use super::{
    contracts::{ResourcePreparationProgress, ResourcePreparationStage},
    coordinator::{complete, Installation, InstallationState},
    locks::{resource_lock, ModelIdentity},
};
use crate::local_resources::models;

pub(super) fn start<R: Runtime>(
    app: AppHandle<R>,
    identity: ModelIdentity,
    installation: Arc<Installation>,
) {
    tauri::async_runtime::spawn(async move {
        let outcome = run(&app, identity, &installation).await;
        let state = match outcome {
            Ok(()) => InstallationState::Progress(ResourcePreparationProgress {
                stage: ResourcePreparationStage::Ready,
                downloaded: identity.snapshot.total_bytes(),
                total: Some(identity.snapshot.total_bytes()),
                percent: Some(100),
            }),
            Err(error) => InstallationState::Failed(error.into()),
        };
        complete(identity, &installation, state);
    });
}

async fn run<R: Runtime>(
    app: &AppHandle<R>,
    identity: ModelIdentity,
    installation: &Installation,
) -> Result<(), String> {
    if models::installation::ready(app, identity.snapshot)? {
        return Ok(());
    }
    let lock = resource_lock(identity.snapshot)?;
    let _exclusive = tokio::select! {
        biased;
        _ = installation.cancellation.cancelled() => {
            return Err("Model preparation cancelled".to_string());
        }
        guard = lock.write_owned() => guard,
    };
    if models::installation::ready(app, identity.snapshot)? {
        return Ok(());
    }
    models::installation::install(
        app,
        identity.snapshot,
        identity.region,
        &mut |update| {
            installation.publish(InstallationState::Progress(update));
        },
        &installation.cancellation,
    )
    .await
}
