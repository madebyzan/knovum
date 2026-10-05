use std::collections::{HashMap, HashSet};

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::models::catalog::{self, ModelKind};

pub(crate) mod commands;
pub(crate) mod contracts;
mod coordinator;
mod installation;
pub(crate) mod locks;
pub(crate) mod requests;

use contracts::{ResourcePreparationProgress, ResourcePreparationStage, ResourcePreparationTarget};

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
    if kinds.is_empty() {
        return Err("Model preparation requires at least one model".to_string());
    }
    if kinds.iter().copied().collect::<HashSet<_>>().len() != kinds.len() {
        return Err("Model preparation contains duplicate model parts".to_string());
    }
    let mut waiters = Vec::with_capacity(kinds.len());
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
        if !status.verified {
            waiters.push((
                kind,
                coordinator::join_installation(app, kind, request_cancellation).await?,
            ));
        }
    }

    let total = waiters
        .iter()
        .map(|(kind, _)| catalog::descriptor(*kind).download_size)
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

pub async fn prepare_models_for_job<R: Runtime>(
    app: &AppHandle<R>,
    target: ResourcePreparationTarget,
    job_id: &str,
    title: &str,
    business_cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<PreparedModels, String> {
    if job_id.trim().is_empty() {
        return Err("Model preparation requires a job identifier".to_string());
    }
    if title.trim().is_empty() {
        return Err("Model preparation requires a task title".to_string());
    }
    let kinds = target
        .models()
        .ok_or_else(|| "Resource preparation target does not contain models".to_string())?;
    let request = requests::begin(job_id, title, target)?;
    prepare_models_for_request(app, kinds, request, business_cancellation, progress).await
}

pub async fn prepare_kokoro_for_job<R: Runtime>(
    app: &AppHandle<R>,
    job_id: &str,
    title: &str,
    business_cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<PreparedKokoroModel, String> {
    if job_id.trim().is_empty() {
        return Err("Kokoro preparation requires a job identifier".to_string());
    }
    if title.trim().is_empty() {
        return Err("Kokoro preparation requires a task title".to_string());
    }
    let request = requests::begin(job_id, title, ResourcePreparationTarget::SpeechSynthesis)?;
    prepare_kokoro_for_request(app, request, business_cancellation, progress).await
}

pub async fn prepare_voxcpm2_for_job<R: Runtime>(
    app: &AppHandle<R>,
    job_id: &str,
    title: &str,
    business_cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<PreparedVoxCpm2Model, String> {
    if job_id.trim().is_empty() {
        return Err("VoxCPM2 preparation requires a job identifier".to_string());
    }
    if title.trim().is_empty() {
        return Err("VoxCPM2 preparation requires a task title".to_string());
    }
    let request = requests::begin(job_id, title, ResourcePreparationTarget::SpeechSynthesis)?;
    let models = prepare_models_for_request(
        app,
        catalog::VOXCPM2_MODELS,
        request,
        business_cancellation,
        progress,
    )
    .await?;
    PreparedVoxCpm2Model::from_models(models)
}

pub(crate) async fn prepare_kokoro_for_request<R: Runtime>(
    app: &AppHandle<R>,
    request: requests::PreparationRequest,
    business_cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<PreparedKokoroModel, String> {
    let prepared = {
        let mut observe = |update: ResourcePreparationProgress| {
            request.update(update.clone());
            progress(update);
        };
        tokio::select! {
            biased;
            _ = business_cancellation.cancelled() => {
                request.cancellation().cancel();
                Err(RESOURCE_PREPARATION_CANCELLED.to_string())
            }
            outcome = prepare_kokoro(app, request.cancellation(), &mut observe) => outcome,
        }
    };
    request.finish(prepared.as_ref().err())?;
    prepared
}

async fn prepare_kokoro<R: Runtime>(
    app: &AppHandle<R>,
    cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<PreparedKokoroModel, String> {
    {
        let _installation = locks::acquire_kokoro_installation(cancellation).await?;
        super::models::kokoro::install(app, progress, cancellation).await?;
    }
    locks::acquire_verified_kokoro(app, cancellation).await
}

pub(crate) async fn prepare_models_for_request<R: Runtime>(
    app: &AppHandle<R>,
    kinds: &'static [ModelKind],
    request: requests::PreparationRequest,
    business_cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<PreparedModels, String> {
    let prepared_models = {
        let mut observe = |update: ResourcePreparationProgress| {
            request.update(update.clone());
            progress(update);
        };
        tokio::select! {
            biased;
            _ = business_cancellation.cancelled() => {
                request.cancellation().cancel();
                Err(RESOURCE_PREPARATION_CANCELLED.to_string())
            }
            prepared_models = prepare_models(app, kinds, request.cancellation(), &mut observe) => prepared_models,
        }
    };
    request.finish(prepared_models.as_ref().err())?;
    prepared_models
}
