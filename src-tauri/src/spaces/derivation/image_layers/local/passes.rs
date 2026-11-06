//! The model passes layer splitting runs over a source. Each pass runs the model a person selected
//! for its capability and owns that model's preparation, so a missing model surfaces as that pass
//! failing, not as a half-built stack.

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
    local_resources::{
        model_catalog::{LocalModelCapability, LocalModelId},
        model_selection,
        models::snapshots::SnapshotKind,
        preparation,
    },
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

/// The model each pass runs, read once so one split never mixes two choices of one capability.
pub(super) struct Models {
    saliency: LocalModelId,
    selection: LocalModelId,
    depth: LocalModelId,
    inpainting: LocalModelId,
}

impl Models {
    pub(super) fn selected(app: &AppHandle) -> Result<Self, String> {
        use LocalModelCapability as C;
        Ok(Self {
            saliency: model_selection::image(app, C::ImageSegmentation)?,
            selection: model_selection::image(app, C::ImageSelection)?,
            depth: model_selection::image(app, C::ImageDepth)?,
            inpainting: model_selection::image(app, C::ImageInpainting)?,
        })
    }
}

/// What every pass of one split is run under.
pub(super) struct Pass<'a> {
    pub(super) app: &'a AppHandle,
    pub(super) models: &'a Models,
    pub(super) owner_id: &'a str,
    pub(super) revision: &'a str,
    pub(super) cancellation: &'a CancellationToken,
}

/// A float field together with the resolution it was produced at.
pub(super) struct Field {
    pub(super) values: Vec<f32>,
    pub(super) width: u32,
    pub(super) height: u32,
}

impl Pass<'_> {
    /// Soft foreground probability, upsampled to the source so it can serve as a subject alpha.
    pub(super) async fn saliency(&self, source: &RgbaImage) -> Result<Vec<u8>, AppError> {
        let model = self.models.saliency;
        let resources = self.prepare(model).await?;
        let identity = ImageSegmentationExecutionIdentity {
            model_spec_id: model.image_segmentation_spec()?,
            owner_kind: "space-derivation".into(),
            owner_id: self.owner_id.into(),
            attempt_id: self.owner_id.into(),
            attempt_epoch: 1,
            input_revision: self.revision.into(),
            budget_ms: SEGMENTATION_BUDGET_MS,
        };
        let edge = crate::images::SALIENCY_EDGE as u32;
        let input = crate::images::saliency_input(&image::DynamicImage::ImageRgba8(source.clone()));
        let field = crate::ai::local_runtime::segment_image(
            self.app,
            input,
            &identity,
            &resources,
            self.cancellation,
        )
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
        &self,
        source: &RgbaImage,
        seeds: Vec<ImageSelectionPoint>,
        max_layers: u8,
    ) -> Result<Vec<u8>, AppError> {
        let tensor = self
            .compute(
                self.models.selection,
                image_tensors::selection_input(source),
                ImageTensorOperation::DiscoverLayers { seeds, max_layers },
            )
            .await?;
        let ranks = GrayImage::from_raw(tensor.shape[3], tensor.shape[2], tensor.bytes)
            .ok_or("Invalid layer discovery label map")?;
        Ok(
            imageops::resize(&ranks, source.width(), source.height(), FilterType::Nearest)
                .into_raw(),
        )
    }

    /// Relative depth at the resolution the depth processor returned.
    pub(super) async fn depth(&self, source: &RgbaImage) -> Result<Field, AppError> {
        let tensor = self
            .compute(
                self.models.depth,
                image_tensors::depth_tensor(source)?,
                ImageTensorOperation::Depth,
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

    /// Paints `holes` (`255` = complete this pixel) from their surroundings. The completed
    /// backdrop keeps the source's own transparency, so a cut-out source yields a cut-out backdrop.
    pub(super) async fn complete(
        &self,
        source: &RgbaImage,
        holes: Vec<u8>,
    ) -> Result<RgbaImage, AppError> {
        let tensor = self
            .compute(
                self.models.inpainting,
                image_tensors::rgb_tensor(source),
                ImageTensorOperation::Inpaint(ImageTensor {
                    bytes: holes,
                    shape: [1, 1, source.height(), source.width()],
                    data_type: TensorDataType::Uint8,
                }),
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
                    source.get_pixel(x, y)[3],
                ])
            },
        ))
    }

    async fn prepare(&self, model: LocalModelId) -> Result<preparation::PreparedModels, AppError> {
        preparation::prepare_models_for_job(
            self.app,
            model.preparation(),
            self.owner_id,
            "Split image layers",
            self.cancellation,
            &mut |_| {},
        )
        .await
        .map_err(Into::into)
    }

    async fn compute(
        &self,
        model: LocalModelId,
        input: ImageTensor,
        operation: ImageTensorOperation,
    ) -> Result<ImageTensor, AppError> {
        let Some(SnapshotKind::Image(variant)) = model.snapshot() else {
            return Err("Layer splitting requires a complete image snapshot".into());
        };
        let resources = self.prepare(model).await?;
        Ok(compute::execute(
            self.app,
            ImageComputeRequest {
                model: variant,
                owner_id: self.owner_id.into(),
                input_revision: self.revision.into(),
                budget_ms: COMPUTE_BUDGET_MS,
                input,
                operation,
            },
            &resources,
            self.cancellation,
        )
        .await?)
    }
}
