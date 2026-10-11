use std::collections::{HashMap, HashSet};

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::models::catalog::ModelKind;

pub(crate) mod commands;
pub(crate) mod contracts;
mod coordinator;
mod installation;
mod jobs;
mod model_plan;
pub(crate) use jobs::{
    prepare_models_for_job, prepare_models_for_request, prepare_voxcpm2_for_job,
};
pub(crate) mod locks;
pub(crate) mod requests;
pub(crate) mod shared_installation;

use contracts::{ResourcePreparationProgress, ResourcePreparationStage};

pub use locks::{
    acquire_verified_models, PreparedDiarizationModel, PreparedModels, PreparedVoxCpm2Model,
};

pub const RESOURCE_PREPARATION_CANCELLED: &str = "local-resource-preparation-cancelled";

pub async fn prepare_models<R: Runtime>(
    app: &AppHandle<R>,
    kinds: &[ModelKind],
    request_cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<PreparedModels, String> {
    if kinds.is_empty() {
        return Err("Model preparation requires at least one model".to_string());
    }
    if kinds.iter().copied().collect::<HashSet<_>>().len() != kinds.len() {
        return Err("Model preparation contains duplicate model parts".to_string());
    }
    let handle = app.clone();
    let requested = kinds.to_vec();
    let plan =
        crate::foundation::blocking::run(move || model_plan::inspect(&handle, &requested)).await?;
    let total = plan.download_bytes;
    progress(ResourcePreparationProgress {
        stage: ResourcePreparationStage::Queued,
        downloaded: 0,
        total,
        percent: total.map(|_| 0),
    });
    let mut waiters = Vec::with_capacity(plan.missing.len());
    for kind in plan.missing {
        waiters.push((
            kind,
            coordinator::join_installation(app, kind, request_cancellation).await?,
        ));
    }
    let mut downloaded = HashMap::<ModelKind, u64>::new();
    for (kind, waiter) in &mut waiters {
        waiter
            .wait(request_cancellation, &mut |update| {
                downloaded.insert(*kind, update.downloaded);
                let current = downloaded.values().copied().sum::<u64>();
                let stage = if matches!(update.stage, ResourcePreparationStage::Ready) {
                    ResourcePreparationStage::Verifying
                } else {
                    update.stage
                };
                progress(ResourcePreparationProgress {
                    stage,
                    downloaded: current,
                    total,
                    percent: total
                        .filter(|total| *total > 0)
                        .map(|total| ((current.saturating_mul(100) / total).min(100)) as u8),
                });
            })
            .await?;
    }

    let prepared = locks::acquire_verified_models(app, kinds, request_cancellation).await?;
    progress(ResourcePreparationProgress {
        stage: ResourcePreparationStage::Ready,
        downloaded: downloaded.values().copied().sum::<u64>(),
        total,
        percent: Some(100),
    });
    Ok(prepared)
}
