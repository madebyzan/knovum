mod assets;
mod contracts;
mod fal;
mod media;
mod persistence;
mod registry;
mod source;

use tauri::{ipc::Channel, AppHandle};

use super::CompletedDerivationOutput;
use crate::app_error::AppError;
pub use contracts::{GenerateImageLayersRequest, ImageLayerAvailability};

pub(crate) async fn availability() -> ImageLayerAvailability {
    use contracts::ImageLayerBlockReason;
    match crate::secrets::read_secret(contracts::FAL_SECRET_KEY).await {
        Ok(Some(secret)) if !secret.trim().is_empty() => ImageLayerAvailability::Ready,
        Ok(_) => ImageLayerAvailability::Blocked {
            reason: ImageLayerBlockReason::MissingCredential,
        },
        Err(_) => ImageLayerAvailability::Blocked {
            reason: ImageLayerBlockReason::SecretDatabaseUnavailable,
        },
    }
}

pub async fn generate(
    app: AppHandle,
    request: GenerateImageLayersRequest,
    on_stage: Channel<String>,
) -> Result<Vec<CompletedDerivationOutput>, AppError> {
    contracts::validate(&request)?;
    let api_key = require_api_key().await?;
    let (generation, _lease) = registry::acquire(&request.generation_id)?;
    let cancellation = generation.cancellation();
    persistence::start(&app, &request).await?;
    let encode_source_image_uri = match source::load_image_uri(&app, &request).await {
        Ok(data_uri) => data_uri,
        Err(error) => return failed(&app, &request, error.into()).await,
    };
    stage(&on_stage, "submitting");
    stage(&on_stage, "processing");
    let layers = match fal::split(
        &api_key,
        &encode_source_image_uri,
        request.output_node_ids.len(),
        request.prompt.as_deref(),
        cancellation,
        || stage(&on_stage, "downloading"),
    )
    .await
    {
        Ok(layers) => layers,
        Err(error) => return failed(&app, &request, error.into()).await,
    };
    let preparation_app = app.clone();
    let preparation_cancellation = cancellation.clone();
    let prepared = match crate::blocking::run(move || {
        assets::prepare(&preparation_app, layers, &preparation_cancellation)
    })
    .await
    {
        Ok(prepared) => prepared,
        Err(error) => return failed(&app, &request, error.into()).await,
    };
    stage(&on_stage, "publishing");
    match persistence::complete(&app, &request, &prepared, &generation).await {
        Ok(completed) => Ok(completed),
        Err(error) => {
            assets::remove(&prepared);
            failed(&app, &request, error).await
        }
    }
}

pub fn cancel(generation_id: &str) -> Result<(), String> {
    registry::cancel(generation_id)
}

async fn require_api_key() -> Result<String, AppError> {
    crate::secrets::read_secret(contracts::FAL_SECRET_KEY)
        .await?
        .filter(|secret| !secret.trim().is_empty())
        .ok_or_else(|| "Image layer API credential is missing".into())
}

async fn failed<T>(
    app: &AppHandle,
    request: &GenerateImageLayersRequest,
    error: AppError,
) -> Result<T, AppError> {
    persistence::fail(app, request, &error.message).await;
    Err(error)
}

fn stage(channel: &Channel<String>, value: &str) {
    let _ = channel.send(value.to_owned());
}
