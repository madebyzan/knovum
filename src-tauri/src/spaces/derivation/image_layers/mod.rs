//! Splits one image into layer nodes. The layer count is a property of the image rather than an
//! input, so the Action reserves no output up front: discovery reports how many layers the source
//! carries and the split reserves exactly that many pending nodes before publishing into them.

mod assets;
mod contracts;
mod local;
mod persistence;
mod source;

use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use super::{action_execution::ExecutionAuthority, CompletedDerivationOutput};
use crate::foundation::app_error::AppError;
pub use contracts::SplitImageLayersRequest;

pub(crate) async fn split_for_action(
    app: &AppHandle,
    request: SplitImageLayersRequest,
    authority: &ExecutionAuthority,
    cancellation: &CancellationToken,
) -> Result<Vec<CompletedDerivationOutput>, AppError> {
    contracts::validate(&request)?;
    let source = source::load(crate::spaces::db::pool(app).await?, &request).await?;
    let png = source::render_png(app, &source, &request).await?;
    // The backdrop can be one of the published layers, so discovery stays one below the ceiling.
    let subjects = u8::try_from(contracts::MAX_LAYER_COUNT - 1)
        .map_err(|_| AppError::from("Image layer ceiling is misconfigured"))?;
    let layers = local::split(app, png, &request.id, subjects, cancellation).await?;
    contracts::validate_count(layers.len())?;
    let prepared = {
        let app = app.clone();
        let cancellation = cancellation.clone();
        crate::foundation::blocking::run(move || assets::prepare(&app, layers, &cancellation))
            .await?
    };
    match publish(app, &request, &source, &prepared, authority).await {
        Ok(outputs) => Ok(outputs),
        Err(error) => {
            assets::remove(&prepared);
            Err(error)
        }
    }
}

async fn publish(
    app: &AppHandle,
    request: &SplitImageLayersRequest,
    source: &source::SourceImage,
    layers: &[assets::PreparedLayer],
    authority: &ExecutionAuthority,
) -> Result<Vec<CompletedDerivationOutput>, AppError> {
    persistence::reserve(app, request, source, layers, authority).await?;
    if let Err(error) = persistence::start(app, request, layers.len(), authority).await {
        persistence::fail_reserved(app, request, layers.len()).await?;
        return Err(error);
    }
    match persistence::complete(app, request, layers, authority).await {
        Ok(completed) => Ok(completed),
        Err(error) => {
            persistence::fail(app, request, layers.len(), &error.message)
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
