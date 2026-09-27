use std::sync::Arc;

use tauri::{AppHandle, Runtime};

use super::{
    contracts::{ResourcePreparationProgress, ResourcePreparationStage},
    coordinator::{complete, Installation, InstallationState},
    locks::{resource_lock, ModelIdentity},
};
use crate::local_resources::models::{
    self,
    catalog::{self, ModelKind},
};

pub(super) fn start<R: Runtime>(
    app: AppHandle<R>,
    kind: ModelKind,
    identity: ModelIdentity,
    installation: Arc<Installation>,
) {
    tauri::async_runtime::spawn(async move {
        let outcome = run(&app, kind, identity, &installation).await;
        let state = match outcome {
            Ok(()) => InstallationState::Progress(ResourcePreparationProgress {
                stage: ResourcePreparationStage::Ready,
                downloaded: catalog::descriptor(kind).download_size,
                total: Some(catalog::descriptor(kind).download_size),
                percent: Some(100),
            }),
            Err(error) => InstallationState::Failed(error.into()),
        };
        complete(identity, &installation, state);
    });
}

async fn run<R: Runtime>(
    app: &AppHandle<R>,
    kind: ModelKind,
    identity: ModelIdentity,
    installation: &Installation,
) -> Result<(), String> {
    if models::status(app, catalog::descriptor(kind))?.verified {
        return Ok(());
    }
    let lock = resource_lock(identity)?;
    let _exclusive = tokio::select! {
        biased;
        _ = installation.cancellation.cancelled() => {
            return Err("Model preparation cancelled".to_string());
        }
        guard = lock.write_owned() => guard,
    };
    if models::status(app, catalog::descriptor(kind))?.verified {
        return Ok(());
    }
    models::install(
        app,
        catalog::descriptor(kind),
        &mut |update| {
            installation.publish(InstallationState::Progress(update));
        },
        &installation.cancellation,
    )
    .await
}
