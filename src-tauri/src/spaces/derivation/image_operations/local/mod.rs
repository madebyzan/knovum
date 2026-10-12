mod background;
mod raster;
mod selection;

use super::{contracts::ImageOperation, output::CanonicalImage, source::SourceImage};
use crate::{
    ai::local_runtime::image::compute::{
        self, ImageComputeRequest, ImageTensor, ImageTensorOperation,
    },
    foundation::app_error::AppError,
    local_resources::{model_catalog::LocalModelId, models::snapshots::SnapshotKind, preparation},
};
pub(super) use selection::preview;
use sha2::Digest;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

/// Upscaling, inpainting, selection and depth all run a single pass over one staged tensor. The
/// budget covers that pass; loading the model is bounded by the worker's own startup budget.
const COMPUTE_BUDGET_MS: u64 = 20 * 60 * 1_000;

pub(super) async fn execute(
    app: &AppHandle,
    model: LocalModelId,
    operation: &ImageOperation,
    source: &SourceImage,
    owner_id: &str,
    cancellation: &CancellationToken,
) -> Result<CanonicalImage, AppError> {
    if matches!(operation, ImageOperation::RemoveBackground) {
        return background::execute(app, model, source, owner_id, cancellation).await;
    }
    let (source_image, input, tensor_operation) = match operation {
        ImageOperation::ExtractSelection { selection } => {
            let (source_image, mask) =
                selection::compute(app, model, selection, source, owner_id, cancellation).await?;
            let operation = operation.clone();
            let token = cancellation.clone();
            return crate::foundation::blocking::run(move || {
                raster::materialize(source_image, mask, &operation, &token)
            })
            .await
            .map_err(Into::into);
        }
        ImageOperation::RemoveSelectedObject {
            selection,
            selection_model,
            selection_spec_id,
        } => {
            if selection_model.image_spec()? != *selection_spec_id {
                return Err("Selection model specification changed".into());
            }
            let (source_image, mask) = selection::compute(
                app,
                *selection_model,
                selection,
                source,
                owner_id,
                cancellation,
            )
            .await?;
            let input = raster::rgb_tensor(&source_image);
            (source_image, input, ImageTensorOperation::Inpaint(mask))
        }
        _ => {
            let source_bytes = source.bytes.clone();
            let prepared_operation = operation.clone();
            let token = cancellation.clone();
            crate::foundation::blocking::run(move || {
                raster::prepare(&source_bytes, &prepared_operation, &token)
            })
            .await?
        }
    };
    let pixels = infer(
        app,
        model,
        source,
        owner_id,
        input,
        tensor_operation,
        cancellation,
    )
    .await?;
    let operation = operation.clone();
    let token = cancellation.clone();
    crate::foundation::blocking::run(move || {
        raster::materialize(source_image, pixels, &operation, &token)
    })
    .await
    .map_err(Into::into)
}

async fn infer(
    app: &AppHandle,
    model: LocalModelId,
    source: &SourceImage,
    owner_id: &str,
    input: ImageTensor,
    operation: ImageTensorOperation,
    cancellation: &CancellationToken,
) -> Result<ImageTensor, AppError> {
    let Some(SnapshotKind::Image(variant)) = model.snapshot() else {
        return Err("Image operation requires a complete image snapshot".into());
    };
    let prepared = preparation::prepare_models_for_job(
        app,
        model.preparation(),
        owner_id,
        "Image processing",
        cancellation,
        &mut |_| {},
    )
    .await?;
    Ok(compute::execute(
        app,
        ImageComputeRequest {
            model: variant,
            owner_id: owner_id.into(),
            input_revision: format!(
                "sha256:{}",
                hex::encode(sha2::Sha256::digest(&source.bytes))
            ),
            budget_ms: COMPUTE_BUDGET_MS,
            input,
            operation,
        },
        &prepared,
        cancellation,
    )
    .await?)
}

#[cfg(test)]
mod tests;
