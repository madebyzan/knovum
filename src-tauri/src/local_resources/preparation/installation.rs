use std::sync::Arc;

use tauri::{AppHandle, Runtime};

use super::{
    contracts::{ResourcePreparationProgress, ResourcePreparationStage},
    coordinator::{complete, Installation, InstallationState},
    locks::resource_lock,
};
use crate::local_resources::models::{self, snapshots::SnapshotKind};

pub(super) fn start<R: Runtime>(
    app: AppHandle<R>,
    snapshot: SnapshotKind,
    installation: Arc<Installation>,
) {
    tauri::async_runtime::spawn(async move {
        let outcome = run(&app, snapshot, &installation).await;
        let state = match outcome {
            Ok(()) => InstallationState::Progress(ResourcePreparationProgress {
                stage: ResourcePreparationStage::Ready,
                downloaded: snapshot.total_bytes(),
                total: Some(snapshot.total_bytes()),
                percent: Some(100),
            }),
            Err(error) => InstallationState::Failed(error.into()),
        };
        complete(snapshot, &installation, state);
    });
}

async fn run<R: Runtime>(
    app: &AppHandle<R>,
    snapshot: SnapshotKind,
    installation: &Installation,
) -> Result<(), String> {
    if models::installation::ready(app, snapshot)? {
        return Ok(());
    }
    let lock = resource_lock(snapshot)?;
    let _exclusive = tokio::select! {
        biased;
        _ = installation.cancellation.cancelled() => {
            return Err("Model preparation cancelled".to_string());
        }
        guard = lock.write_owned() => guard,
    };
    if models::installation::ready(app, snapshot)? {
        return Ok(());
    }
    models::installation::install(
        app,
        snapshot,
        &mut |update| {
            installation.publish(InstallationState::Progress(update));
        },
        &installation.cancellation,
    )
    .await
}
