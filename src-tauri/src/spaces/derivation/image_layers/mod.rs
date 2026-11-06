//! Separates one picture into the layers it is composed of: the subjects that stand in it, ordered
//! far to near, over the backdrop they stood in front of with the space they covered completed.
//! The image decides how many layers it carries, so the set is published through [`image_set`].
//!
//! [`image_set`]: super::image_set

mod local;

use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use super::{
    action_execution::ExecutionAuthority,
    image_set::{self, ImageSet, ImageSetRequest},
    CompletedDerivationOutput,
};
use crate::foundation::app_error::AppError;

/// Layers a split may publish: the subjects discovery can rank, and the backdrop beneath them.
const MAX_LAYERS: usize = crate::ai::contracts::image_compute::MAX_DISCOVERED_LAYERS as usize + 1;

pub(crate) static SET: ImageSet = ImageSet {
    action_kind: "split-image-layers",
    engine: "local:image-layers",
    lineage: "Image layer separated from source image",
    member: "layer",
    // A stack needs something to stand over something else.
    members: 2..=MAX_LAYERS,
    unsupported: "This image did not separate into layers",
};

pub(crate) async fn split_for_action(
    app: &AppHandle,
    request: ImageSetRequest,
    authority: &ExecutionAuthority,
    cancellation: &CancellationToken,
) -> Result<Vec<CompletedDerivationOutput>, AppError> {
    let owner = request.id.clone();
    image_set::derive(app, &SET, request, authority, cancellation, |png| {
        local::split(app, png, &owner, cancellation)
    })
    .await
}
