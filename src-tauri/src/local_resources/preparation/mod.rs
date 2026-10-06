use std::collections::{HashMap, HashSet};

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::models::catalog::{self, ModelKind};

pub(crate) mod commands;
pub(crate) mod contracts;
mod coordinator;
mod installation;
mod jobs;
pub(crate) use jobs::{
    prepare_kokoro_for_job, prepare_kokoro_for_request, prepare_models_for_job,
    prepare_models_for_request, prepare_python_for_request, prepare_voxcpm2_for_job,
};
pub(crate) mod locks;
pub(crate) mod requests;
pub(crate) mod shared_installation;

use contracts::{ResourcePreparationProgress, ResourcePreparationStage};

pub use locks::{
    acquire_verified_diarization, acquire_verified_models, PreparedDiarizationModel,
    PreparedKokoroModel, PreparedModels, PreparedVoxCpm2Model,
};

pub const RESOURCE_PREPARATION_CANCELLED: &str = "local-resource-preparation-cancelled";

pub async fn prepare_models<R: Runtime>(
    app: &AppHandle<R>,
    kinds: &[ModelKind],
    request_cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<PreparedModels, String> {
    super::runtimes::python::prepare(app, request_cancellation, &mut |update| {
        if !matches!(update.stage, ResourcePreparationStage::Ready) {
            progress(update);
        }
    })
    .await?;
    if kinds.is_empty() {
        return Err("Model preparation requires at least one model".to_string());
    }
    if kinds.iter().copied().collect::<HashSet<_>>().len() != kinds.len() {
        return Err("Model preparation contains duplicate model parts".to_string());
    }
    let mut waiters = Vec::with_capacity(kinds.len());
    let mut snapshots = HashSet::new();
    for &kind in kinds {
        let descriptor = catalog::descriptor(kind);
        let status = super::models::status(app, descriptor)?;
        if !status.verified
            && super::configured_path::configured_path(descriptor.env_key)?.is_some()
        {
            return Err(format!(
                "Configured model {} failed verification",
                descriptor.file
            ));
        }
        if !status.verified && snapshots.insert(descriptor.snapshot) {
            waiters.push((
                kind,
                coordinator::join_installation(app, kind, request_cancellation).await?,
            ));
        }
    }

    let total = waiters
        .iter()
        .map(|(kind, _)| catalog::descriptor(*kind).snapshot.total_bytes())
        .sum::<u64>();
    if waiters.is_empty() {
        progress(ResourcePreparationProgress {
            stage: ResourcePreparationStage::Ready,
            downloaded: 0,
            total: Some(0),
            percent: Some(100),
        });
    }
    let waiter_count = waiters.len();
    let mut downloaded = HashMap::<ModelKind, u64>::new();
    for (index, (kind, waiter)) in waiters.iter_mut().enumerate() {
        waiter
            .wait(request_cancellation, &mut |update| {
                downloaded.insert(*kind, update.downloaded);
                let current = downloaded.values().copied().sum::<u64>();
                let stage = if matches!(update.stage, ResourcePreparationStage::Ready)
                    && index + 1 < waiter_count
                {
                    ResourcePreparationStage::Downloading
                } else {
                    update.stage
                };
                progress(ResourcePreparationProgress {
                    stage,
                    downloaded: current,
                    total: Some(total),
                    percent: (total > 0)
                        .then(|| ((current.saturating_mul(100) / total).min(100)) as u8),
                });
            })
            .await?;
    }

    locks::acquire_verified_models(app, kinds, request_cancellation).await
}
