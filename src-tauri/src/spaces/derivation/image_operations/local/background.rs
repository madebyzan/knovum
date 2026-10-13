use super::super::{output::CanonicalImage, source::SourceImage};
use crate::{
    ai::contracts::image_wire::ImageSegmentationExecutionIdentity,
    foundation::app_error::AppError,
    local_resources::{model_catalog::LocalModelId, preparation},
};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

/// Background removal runs one segmentation pass over a fixed-resolution tensor. Loading the model
/// is bounded by the worker's own startup budget.
const SEGMENTATION_BUDGET_MS: u64 = 120_000;

pub(super) async fn execute(
    app: &AppHandle,
    model: LocalModelId,
    source: &SourceImage,
    owner_id: &str,
    cancellation: &CancellationToken,
) -> Result<CanonicalImage, AppError> {
    let resources = preparation::prepare_models_for_job(
        app,
        model.preparation(),
        owner_id,
        "Remove image background",
        cancellation,
        &mut |_| {},
    )
    .await?;
    let identity = ImageSegmentationExecutionIdentity {
        model_spec_id: model.image_segmentation_spec()?,
        owner_kind: "space-derivation".into(),
        owner_id: owner_id.into(),
        attempt_id: owner_id.into(),
        attempt_epoch: 1,
        input_revision: format!(
            "sha256:{}",
            hex::encode(<sha2::Sha256 as sha2::Digest>::digest(&source.bytes))
        ),
        budget_ms: SEGMENTATION_BUDGET_MS,
    };
    let image = crate::images::remove_background(
        app,
        source.bytes.clone(),
        &identity,
        &resources,
        cancellation,
    )
    .await?;
    Ok(CanonicalImage {
        digest: format!(
            "sha256:{}",
            hex::encode(<sha2::Sha256 as sha2::Digest>::digest(&image.png))
        ),
        png: image.png,
        width: image.width,
        height: image.height,
    })
}
