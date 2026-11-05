//! Derivations that publish a set of images whose size the source decides — the layers a picture
//! separates into, the stickers a sheet carries. The count is a property of the image rather than
//! an input, so the Run reserves a single pending node up front like every other derivation — the
//! click lands on the canvas immediately — and the feature then reports what the source carries.
//! The first member publishes into that node and the rest are reserved beside it, so the whole set
//! is laid out together. A feature supplies only its [`ImageSet`] and how a rendered source becomes
//! member PNGs; reserving, opening, settling and failing the set is this module's.

mod assets;
mod contracts;
mod persistence;
mod source;

#[cfg(test)]
mod tests;

use std::future::Future;

use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use super::{action_execution::ExecutionAuthority, failure, CompletedDerivationOutput};
use crate::foundation::app_error::AppError;
pub(crate) use contracts::{ImageSet, ImageSetRequest};

/// Derives `set` from the pinned source. `members` turns the rendered source PNG into the member
/// PNGs in the order they publish; an empty or oversized answer is the set's `unsupported` error.
pub(crate) async fn derive<Members, Read>(
    app: &AppHandle,
    set: &'static ImageSet,
    request: ImageSetRequest,
    authority: &ExecutionAuthority,
    cancellation: &CancellationToken,
    members: Members,
) -> Result<Vec<CompletedDerivationOutput>, AppError>
where
    Members: FnOnce(Vec<u8>) -> Read,
    Read: Future<Output = Result<Vec<Vec<u8>>, AppError>>,
{
    match derive_set(app, set, &request, authority, cancellation, members).await {
        Ok(outputs) => Ok(outputs),
        Err(error) => {
            // Until the derivations open, the node the Run reserved is the only record of the set
            // on the canvas, so it carries the failure. Settling only acts on a node that is still
            // pending, so a set that already settled its own outputs passes through.
            let settlement = failure::fail_pending(app, &request.output_node_id).await;
            Err(failure::with_settlement(error, settlement))
        }
    }
}

async fn derive_set<Members, Read>(
    app: &AppHandle,
    set: &'static ImageSet,
    request: &ImageSetRequest,
    authority: &ExecutionAuthority,
    cancellation: &CancellationToken,
    members: Members,
) -> Result<Vec<CompletedDerivationOutput>, AppError>
where
    Members: FnOnce(Vec<u8>) -> Read,
    Read: Future<Output = Result<Vec<Vec<u8>>, AppError>>,
{
    contracts::validate(request)?;
    let source = source::load(crate::spaces::db::pool(app).await?, request).await?;
    let png = source::render_png(app, &source, request).await?;
    let encoded = members(png).await?;
    set.validate_count(encoded.len())?;
    let prepared = {
        let app = app.clone();
        let cancellation = cancellation.clone();
        crate::foundation::blocking::run(move || assets::prepare(&app, set, encoded, &cancellation))
            .await?
    };
    match publish(app, set, request, &source, &prepared, authority).await {
        Ok(outputs) => Ok(outputs),
        Err(error) => {
            assets::remove(&prepared);
            Err(error)
        }
    }
}

async fn publish(
    app: &AppHandle,
    set: &ImageSet,
    request: &ImageSetRequest,
    source: &source::SourceImage,
    members: &[assets::PreparedMember],
    authority: &ExecutionAuthority,
) -> Result<Vec<CompletedDerivationOutput>, AppError> {
    persistence::reserve(app, set, request, source, members, authority).await?;
    if let Err(error) = persistence::start(app, set, request, members.len(), authority).await {
        persistence::fail_reserved(app, set, request, members.len()).await?;
        return Err(error);
    }
    match persistence::complete(app, set, request, members, authority).await {
        Ok(completed) => Ok(completed),
        Err(error) => {
            persistence::fail(app, set, request, members.len(), &error.message)
                .await
                .map_err(|settlement| {
                    AppError::new(
                        "derivation_settlement_failed",
                        format!(
                            "{}; Failed to persist the image set failure: {settlement}",
                            error.message
                        ),
                    )
                })?;
            Err(error)
        }
    }
}
