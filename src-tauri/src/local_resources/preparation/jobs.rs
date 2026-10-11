use super::{
    contracts::{ResourcePreparationProgress, ResourcePreparationTarget},
    prepare_models, requests, PreparedModels, PreparedVoxCpm2Model, RESOURCE_PREPARATION_CANCELLED,
};
use crate::local_resources::models::catalog::ModelKind;
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
    let kinds = target
        .models()
        .ok_or_else(|| "Resource preparation target does not contain models".to_string())?;
    let request = requests::begin(job_id, title, target)?;
    prepare_models_for_request(app, kinds, request, business_cancellation, progress).await
}

pub async fn prepare_voxcpm2_for_job<R: Runtime>(
    app: &AppHandle<R>,
    model_spec_id: &str,
    job_id: &str,
    title: &str,
    business_cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<PreparedVoxCpm2Model, String> {
    let model =
        crate::local_resources::model_catalog::LocalModelId::from_voxcpm2_spec(model_spec_id)?;
    let request = requests::begin(job_id, title, model.preparation())?;
    let models = prepare_models_for_request(
        app,
        model.voxcpm2_models()?,
        request,
        business_cancellation,
        progress,
    )
    .await?;
    PreparedVoxCpm2Model::from_models(models, model)
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
