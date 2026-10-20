use super::{depth, discovery, inpainting, segmentation, selection, upscaling};
use crate::{artifact, cancellation::Cancellation};
use knovum_inference_contracts::{
    image_compute::{ImageComputation, ImageComputeParams, ImageModelVariant},
    image_wire::TensorDataType,
    runtime::{RuntimeInitializeParams, RuntimeModel},
};
use std::path::Path;

/// One worker owns exactly one image model, so the loaded engine decides which computations the
/// worker can answer.
pub(super) enum ImageEngine {
    Segmentation(segmentation::Segmentation),
    Upscaling(upscaling::Upscaling),
    Inpainting(inpainting::Inpainting),
    Selection(selection::Selection),
    Depth(depth::Depth),
}

pub(super) struct Computed {
    pub(super) pixels: Vec<u8>,
    pub(super) shape: [u32; 4],
    pub(super) data_type: TensorDataType,
}

impl ImageEngine {
    pub(super) fn load(params: &RuntimeInitializeParams) -> Result<Self, String> {
        let threads = params.device_profile.cpu_threads;
        let (variant, directory) = match &params.model {
            RuntimeModel::Snapshot { path } => {
                return segmentation::Segmentation::new(Path::new(path), threads)
                    .map(ImageEngine::Segmentation)
            }
            RuntimeModel::ImageVariant { path, variant } => (*variant, Path::new(path)),
        };
        use ImageModelVariant as V;
        match variant {
            V::RealEsrganGeneralX4v3 | V::RealEsrganX4plus => {
                upscaling::Upscaling::new(directory, variant == V::RealEsrganX4plus, threads)
                    .map(ImageEngine::Upscaling)
            }
            V::Migan => {
                inpainting::Inpainting::new(directory, threads).map(ImageEngine::Inpainting)
            }
            V::EfficientSamTi | V::EfficientSamS => {
                selection::Selection::new(directory, variant == V::EfficientSamS, threads)
                    .map(ImageEngine::Selection)
            }
            V::DepthAnythingV2SmallUint8 | V::DepthAnythingV2Small => {
                depth::Depth::new(directory, variant == V::DepthAnythingV2SmallUint8, threads)
                    .map(ImageEngine::Depth)
            }
        }
    }

    pub(super) fn compute(
        &mut self,
        work: &Path,
        params: &ImageComputeParams,
        input: &[u8],
        cancel: &Cancellation,
    ) -> Result<Computed, String> {
        let shape = params.tensor_ref.shape;
        let [_, _, height, width] = shape;
        let mask_shape = [1, 1, height, width];
        let identity = || format!("{}:{}", params.input_revision, params.tensor_ref.sha256);
        match (self, &params.computation, params.tensor_ref.data_type) {
            (Self::Upscaling(engine), ImageComputation::Upscale, TensorDataType::Uint8) => {
                Ok(Computed {
                    pixels: engine.execute(shape, input, cancel)?,
                    shape: [1, 3, height * 4, width * 4],
                    data_type: TensorDataType::Uint8,
                })
            }
            (
                Self::Inpainting(engine),
                ImageComputation::Inpaint { mask_ref },
                TensorDataType::Uint8,
            ) => {
                if mask_ref.data_type != TensorDataType::Uint8 || mask_ref.shape != mask_shape {
                    return Err("invalid_input: inpainting mask shape or type differs".into());
                }
                let mask = artifact::read_tensor(work, mask_ref, 16 * 1024 * 1024)?;
                Ok(Computed {
                    pixels: engine.execute(shape, input, &mask, cancel)?,
                    shape,
                    data_type: TensorDataType::Uint8,
                })
            }
            (
                Self::Selection(engine),
                ImageComputation::Select { points, box_rect },
                TensorDataType::Uint8,
            ) => Ok(Computed {
                pixels: engine.execute(
                    shape,
                    input,
                    identity(),
                    points,
                    box_rect.as_ref(),
                    cancel,
                )?,
                shape: mask_shape,
                data_type: TensorDataType::Uint8,
            }),
            (
                Self::Selection(engine),
                ImageComputation::DiscoverLayers { seeds, max_layers },
                TensorDataType::Uint8,
            ) => Ok(Computed {
                pixels: discovery::discover(
                    engine,
                    shape,
                    input,
                    identity(),
                    (seeds, *max_layers),
                    cancel,
                )?,
                shape: mask_shape,
                data_type: TensorDataType::Uint8,
            }),
            (Self::Depth(engine), ImageComputation::Depth, TensorDataType::Float32LittleEndian) => {
                Ok(Computed {
                    pixels: engine.execute(shape, input, cancel)?,
                    shape: mask_shape,
                    data_type: TensorDataType::Float32LittleEndian,
                })
            }
            _ => Err("runtime_mismatch: image operation or input type differs".into()),
        }
    }
}
