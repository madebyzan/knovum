mod assets;
mod contracts;
mod fal;
mod media;
mod persistence;
mod registry;
mod source;

use tauri::{ipc::Channel, AppHandle};

use super::CompletedDerivationOutput;
pub use contracts::{CreateImageLayersRequest, ImageLayerCapability};

pub(crate) fn capability() -> ImageLayerCapability {
    use contracts::ImageLayerBlockReason;
    match crate::secrets::read_secret(contracts::FAL_SECRET_KEY) {
        Ok(Some(secret)) if !secret.trim().is_empty() => ImageLayerCapability::Ready,
        Ok(_) => ImageLayerCapability::Blocked {
            reason: ImageLayerBlockReason::MissingCredential,
        },
        Err(_) => ImageLayerCapability::Blocked {
            reason: ImageLayerBlockReason::SecretVaultUnavailable,
        },
    }
}

pub async fn create(
    app: AppHandle,
    request: CreateImageLayersRequest,
    on_stage: Channel<String>,
) -> Result<Vec<CompletedDerivationOutput>, String> {
    contracts::validate(&request)?;
    let api_key = require_api_key()?;
    let (cancellation, _lease) = registry::acquire(&request.id)?;
    persistence::start(&app, &request).await?;
    let source_data_uri = match source::data_uri(&app, &request).await {
        Ok(data_uri) => data_uri,
        Err(error) => return failed(&app, &request, error).await,
    };
    stage(&on_stage, "submitting");
    stage(&on_stage, "processing");
    let layers = match fal::split(
        &api_key,
        &source_data_uri,
        request.output_node_ids.len(),
        request.prompt.as_deref(),
        &cancellation,
        || stage(&on_stage, "downloading"),
    )
    .await
    {
        Ok(layers) => layers,
        Err(error) => return failed(&app, &request, error).await,
    };
    let prepared = match assets::prepare(&app, layers) {
        Ok(prepared) => prepared,
        Err(error) => return failed(&app, &request, error).await,
    };
    stage(&on_stage, "publishing");
    match persistence::complete(&app, &request, &prepared).await {
        Ok(completed) => Ok(completed),
        Err(error) => {
            assets::remove(&prepared);
            failed(&app, &request, error).await
        }
    }
}

pub fn cancel(id: &str) -> Result<(), String> {
    registry::cancel(id)
}

fn require_api_key() -> Result<String, String> {
    crate::secrets::read_secret(contracts::FAL_SECRET_KEY)?
        .filter(|secret| !secret.trim().is_empty())
        .ok_or_else(|| "Image layer API credential is missing".into())
}

async fn failed<T>(
    app: &AppHandle,
    request: &CreateImageLayersRequest,
    error: String,
) -> Result<T, String> {
    persistence::fail(app, request, &error).await;
    Err(error)
}

fn stage(channel: &Channel<String>, value: &str) {
    let _ = channel.send(value.to_owned());
}
