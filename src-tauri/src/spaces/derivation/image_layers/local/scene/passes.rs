//! The model passes layer discovery runs over a photographic source. Each pass owns its model
//! preparation so a missing model surfaces as that pass failing, not as a half-built stack.

use crate::{
    ai::{
        contracts::{
            image_compute::ImageSelectionPoint,
            image_wire::{ImageSegmentationExecutionIdentity, TensorDataType},
        },
        local_runtime::image::compute::{
            self, ImageComputeRequest, ImageTensor, ImageTensorOperation,
        },
    },
    foundation::app_error::AppError,
    local_resources::{model_catalog::LocalModelId, models::snapshots::SnapshotKind, preparation},
    spaces::derivation::image_tensors,
};
use image::{
    imageops::{self, FilterType},
    GrayImage, Luma, Rgba, RgbaImage,
};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

const SEGMENTATION_BUDGET_MS: u64 = 120_000;
/// Discovery and inpainting run one pass each over a staged tensor; loading the model is bounded
/// by the worker's own startup budget.
const COMPUTE_BUDGET_MS: u64 = 20 * 60 * 1_000;

/// A float field together with the resolution it was produced at.
pub(super) struct Field {
    pub(super) values: Vec<f32>,
    pub(super) width: u32,
    pub(super) height: u32,
}

/// Soft foreground probability, upsampled to the source so it can serve as a subject alpha.
pub(super) async fn saliency(
    app: &AppHandle,
    source: &RgbaImage,
    (owner_id, revision): (&str, &str),
    cancellation: &CancellationToken,
) -> Result<Vec<u8>, AppError> {
    let model = LocalModelId::U2netp;
    let resources = prepare(app, model, owner_id, cancellation).await?;
    let identity = ImageSegmentationExecutionIdentity {
        model_spec_id: model.image_segmentation_spec()?,
        owner_kind: "space-derivation".into(),
        owner_id: owner_id.into(),
        attempt_id: owner_id.into(),
        attempt_epoch: 1,
        input_revision: revision.into(),
        budget_ms: SEGMENTATION_BUDGET_MS,
    };
    let edge = crate::images::SALIENCY_EDGE as u32;
    let input = crate::images::saliency_input(&image::DynamicImage::ImageRgba8(source.clone()));
    let field =
        crate::ai::local_runtime::segment_image(app, input, &identity, &resources, cancellation)
            .await?;
    if field.len() != (edge * edge) as usize {
        return Err("Invalid saliency field".into());
    }
    let field = GrayImage::from_fn(edge, edge, |x, y| {
        Luma([(field[(y * edge + x) as usize].clamp(0.0, 1.0) * 255.0).round() as u8])
    });
    Ok(imageops::resize(
        &field,
        source.width(),
        source.height(),
        FilterType::Triangle,
    )
    .into_raw())
}

/// Discovery ranks, upsampled to the source. Rank `0` is unassigned.
pub(super) async fn discover(
    app: &AppHandle,
    source: &RgbaImage,
    (owner_id, revision): (&str, &str),
    (seeds, max_layers): (Vec<ImageSelectionPoint>, u8),
    cancellation: &CancellationToken,
) -> Result<Vec<u8>, AppError> {
    let tensor = compute(
        app,
        LocalModelId::EfficientSamTi,
        (owner_id, revision),
        image_tensors::selection_input(source),
        ImageTensorOperation::DiscoverLayers { seeds, max_layers },
        cancellation,
    )
    .await?;
    let ranks = GrayImage::from_raw(tensor.shape[3], tensor.shape[2], tensor.bytes)
        .ok_or("Invalid layer discovery label map")?;
    Ok(imageops::resize(&ranks, source.width(), source.height(), FilterType::Nearest).into_raw())
}

/// Relative depth at the resolution the depth processor returned.
pub(super) async fn depth(
    app: &AppHandle,
    source: &RgbaImage,
    (owner_id, revision): (&str, &str),
    cancellation: &CancellationToken,
) -> Result<Field, AppError> {
    let tensor = compute(
        app,
        LocalModelId::DepthAnythingV2Small,
        (owner_id, revision),
        image_tensors::depth_tensor(source)?,
        ImageTensorOperation::Depth,
        cancellation,
    )
    .await?;
    Ok(Field {
        values: tensor
            .bytes
            .chunks_exact(4)
            .map(|bytes| f32::from_le_bytes(bytes.try_into().expect("Validated float tensor")))
            .collect(),
        width: tensor.shape[3],
        height: tensor.shape[2],
    })
}

/// Inpaints every subject away so the backdrop layer is a complete image.
pub(super) async fn complete(
    app: &AppHandle,
    source: &RgbaImage,
    mask: Vec<u8>,
    (owner_id, revision): (&str, &str),
    cancellation: &CancellationToken,
) -> Result<RgbaImage, AppError> {
    let tensor = compute(
        app,
        LocalModelId::Migan,
        (owner_id, revision),
        image_tensors::rgb_tensor(source),
        ImageTensorOperation::Inpaint(ImageTensor {
            bytes: mask,
            shape: [1, 1, source.height(), source.width()],
            data_type: TensorDataType::Uint8,
        }),
        cancellation,
    )
    .await?;
    let count = (source.width() * source.height()) as usize;
    if tensor.bytes.len() != count * 3 {
        return Err("Invalid completed backdrop tensor".into());
    }
    Ok(RgbaImage::from_fn(
        source.width(),
        source.height(),
        |x, y| {
            let index = (y * source.width() + x) as usize;
            Rgba([
                tensor.bytes[index],
                tensor.bytes[count + index],
                tensor.bytes[count * 2 + index],
                255,
            ])
        },
    ))
}

async fn prepare(
    app: &AppHandle,
    model: LocalModelId,
    owner_id: &str,
    cancellation: &CancellationToken,
) -> Result<preparation::PreparedModels, AppError> {
    preparation::prepare_models_for_job(
        app,
        model.preparation(),
        owner_id,
        "Split image layers",
        cancellation,
        &mut |_| {},
    )
    .await
    .map_err(Into::into)
}

async fn compute(
    app: &AppHandle,
    model: LocalModelId,
    (owner_id, revision): (&str, &str),
    input: ImageTensor,
    operation: ImageTensorOperation,
    cancellation: &CancellationToken,
) -> Result<ImageTensor, AppError> {
    let Some(SnapshotKind::Image(variant)) = model.snapshot() else {
        return Err("Layer splitting requires a complete image snapshot".into());
    };
    let resources = prepare(app, model, owner_id, cancellation).await?;
    Ok(compute::execute(
        app,
        ImageComputeRequest {
            model: variant,
            owner_id: owner_id.into(),
            input_revision: revision.into(),
            budget_ms: COMPUTE_BUDGET_MS,
            input,
            operation,
        },
        &resources,
        cancellation,
    )
    .await?)
}
