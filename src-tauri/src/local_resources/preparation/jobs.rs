use super::{
    contracts::{ResourcePreparationProgress, ResourcePreparationTarget},
    locks, prepare_models, requests, PreparedKokoroModel, PreparedModels, PreparedVoxCpm2Model,
    RESOURCE_PREPARATION_CANCELLED,
};
use crate::local_resources::models::catalog::{self, ModelKind};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

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
    use super::contracts::ResourcePreparationStage;
    use crate::local_resources::{models::kokoro, runtimes::python};
    let profile = python::RuntimeProfile::LightInference;
    let handle = app.clone();
    let (model_bytes, runtime_bytes) = crate::blocking::run(move || {
        let model = kokoro::status(&handle)?;
        Ok::<_, String>((
            if model.verified {
                0
            } else {
                model.download_size
            },
            python::download_size(&handle, profile)?,
        ))
    })
    .await?;
    let total = model_bytes + runtime_bytes;
    progress(ResourcePreparationProgress {
        stage: ResourcePreparationStage::Queued,
        downloaded: 0,
        total: Some(total),
        percent: Some(0),
    });
    python::prepare(app, profile, cancellation, &mut |update| {
        if !matches!(update.stage, ResourcePreparationStage::Ready) {
            progress(update.with_overall_progress(0, total));
        }
    })
    .await?;
    {
        let _installation = locks::acquire_kokoro_installation(cancellation).await?;
        kokoro::install(
            app,
            &mut |update| {
                if !matches!(update.stage, ResourcePreparationStage::Ready) {
                    progress(update.with_overall_progress(runtime_bytes, total));
                }
            },
            cancellation,
        )
        .await?;
    }
    let model = locks::acquire_verified_kokoro(app, cancellation).await?;
    progress(ResourcePreparationProgress {
        stage: ResourcePreparationStage::Ready,
        downloaded: total,
        total: Some(total),
        percent: Some(100),
    });
    Ok(model)
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

pub(crate) async fn prepare_agent_for_request<R: Runtime>(
    app: &AppHandle<R>,
    request: requests::PreparationRequest,
    business_cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<(), String> {
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
            outcome = crate::local_resources::runtimes::python::prepare(app,
                crate::local_resources::runtimes::python::RuntimeProfile::Agent,
                request.cancellation(), &mut observe) => outcome,
        }
    };
    request.finish(prepared.as_ref().err())?;
    prepared
}

pub(crate) async fn prepare_agent_for_job<R: Runtime>(
    app: &AppHandle<R>,
    job_id: &str,
    title: &str,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let request = requests::begin(job_id, title, ResourcePreparationTarget::BuiltinAgent)?;
    prepare_agent_for_request(app, request, cancellation, &mut |_| {}).await
}

pub(crate) async fn prepare_diarization_for_request<R: Runtime>(
    app: &AppHandle<R>,
    request: requests::PreparationRequest,
    business_cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<super::PreparedDiarizationModel, String> {
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
            outcome = async {
                use crate::local_resources::{models::diarization, runtimes::python};
                use super::contracts::ResourcePreparationStage;
                let handle = app.clone();
                let (model_bytes, runtime_bytes) = crate::blocking::run(move || {
                    let model = diarization::status(&handle, false)?;
                    if model.installed && !model.verified {
                        return Err("Diarization model is corrupt; remove it before downloading again".into());
                    }
                    Ok::<_, String>((if model.verified { 0 } else { model.total_bytes },
                        python::download_size(&handle, diarization::RUNTIME_PROFILE)?))
                }).await?;
                let total = model_bytes + runtime_bytes;
                observe(ResourcePreparationProgress { stage: ResourcePreparationStage::Queued,
                    downloaded: 0, total: Some(total), percent: Some(0) });
                diarization::prepare(app, request.cancellation(), &mut |update| {
                    if !matches!(update.stage, ResourcePreparationStage::Ready) {
                        observe(update.with_overall_progress(0, total));
                    }
                }).await?;
                crate::local_resources::runtimes::python::prepare(app,
                    diarization::RUNTIME_PROFILE,
                    request.cancellation(), &mut |update| {
                        if !matches!(update.stage, ResourcePreparationStage::Ready) {
                            observe(update.with_overall_progress(model_bytes, total));
                        }
                    }).await?;
                let model = locks::acquire_verified_diarization(app, request.cancellation()).await?;
                observe(ResourcePreparationProgress { stage: ResourcePreparationStage::Ready,
                    downloaded: total, total: Some(total), percent: Some(100) });
                Ok(model)
            } => outcome,
        }
    };
    request.finish(prepared.as_ref().err())?;
    prepared
}

pub(crate) async fn prepare_diarization_for_job<R: Runtime>(
    app: &AppHandle<R>,
    job_id: &str,
    title: &str,
    cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<super::PreparedDiarizationModel, String> {
    let request = requests::begin(job_id, title, ResourcePreparationTarget::SpeechDiarization)?;
    prepare_diarization_for_request(app, request, cancellation, progress).await
}
