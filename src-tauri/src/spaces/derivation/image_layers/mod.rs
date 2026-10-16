mod assets;
mod contracts;
mod local;
mod persistence;
mod plan;
mod registry;
mod source;

use tauri::{ipc::Channel, AppHandle};

use super::CompletedDerivationOutput;
use crate::foundation::app_error::AppError;
pub use contracts::{GenerateImageLayersRequest, ImageLayerPlan, PlanImageLayersRequest};

/// Discovers the layers a source carries and stages them. The backdrop is one of the staged
/// layers, so discovery itself is bounded one below the output ceiling.
pub async fn plan(
    app: AppHandle,
    request: PlanImageLayersRequest,
    on_stage: Channel<String>,
) -> Result<ImageLayerPlan, AppError> {
    contracts::validate_plan(&request)?;
    let (generation, lease) = registry::acquire(&request.plan_id)?;
    stage(&on_stage, "preparing");
    let png = source::load_png(&app, &request.plan_id, &request.source_edit).await?;
    stage(&on_stage, "analyzing");
    let subjects = u8::try_from(contracts::MAX_LAYER_COUNT - 1)
        .map_err(|_| AppError::from("Image layer ceiling is misconfigured"))?;
    let layers = local::split(
        &app,
        png,
        &request.plan_id,
        subjects,
        generation.cancellation(),
    )
    .await?;
    stage(&on_stage, "staging");
    let staging_app = app.clone();
    let staging_cancellation = generation.cancellation().clone();
    let prepared = crate::foundation::blocking::run(move || {
        assets::prepare(&staging_app, layers, &staging_cancellation)
    })
    .await?;
    let layer_count = prepared.len();
    if !(contracts::MIN_LAYER_COUNT..=contracts::MAX_LAYER_COUNT).contains(&layer_count) {
        assets::remove(&prepared);
        return Err("This image did not split into separable layers".into());
    }
    plan::stage(
        &request.plan_id,
        plan::StagedPlan {
            layers: prepared,
            space_id: request.space_id,
            source_node_id: request.source_node_id,
            source_revision: request.source_revision,
            generation,
            lease,
        },
    )?;
    Ok(ImageLayerPlan {
        plan_id: request.plan_id,
        layer_count,
    })
}

/// Publishes a staged plan into the output nodes the caller created for it.
pub async fn generate(
    app: AppHandle,
    request: GenerateImageLayersRequest,
) -> Result<Vec<CompletedDerivationOutput>, AppError> {
    contracts::validate(&request)?;
    let staged = plan::take(&request.plan_id)?;
    if staged.space_id != request.space_id
        || staged.source_node_id != request.source_node_id
        || staged.source_revision != request.source_revision
        || staged.layers.len() != request.output_node_ids.len()
    {
        assets::remove(&staged.layers);
        return Err("Image layer plan does not describe this request".into());
    }
    if let Err(error) = persistence::start(&app, &request).await {
        assets::remove(&staged.layers);
        return Err(error);
    }
    match persistence::complete(&app, &request, &staged.layers, &staged.generation).await {
        Ok(completed) => Ok(completed),
        Err(error) => {
            assets::remove(&staged.layers);
            persistence::fail(&app, &request, &error.message)
                .await
                .map_err(|settlement| {
                    AppError::new(
                        "derivation_settlement_failed",
                        format!(
                            "{}; Failed to persist image layer failure: {settlement}",
                            error.message
                        ),
                    )
                })?;
            Err(error)
        }
    }
}

pub fn cancel(split_id: &str) -> Result<(), String> {
    let cancelled = registry::cancel(split_id);
    plan::discard(split_id);
    cancelled
}

fn stage(channel: &Channel<String>, value: &str) {
    let _ = channel.send(value.to_owned());
}
