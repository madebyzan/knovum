mod depth;
mod inpainting;
mod segmentation;
mod selection;
mod session;
mod upscaling;

use crate::{artifact, cancellation::Cancellation};
use knovum_inference_contracts::{
    image_compute::{
        ImageComputation, ImageComputationOutput, ImageComputeParams, ImageModelVariant,
    },
    image_wire::{ImageSegmentParams, ImageSegmentation, TensorDataType},
    runtime::RuntimeInitializeParams,
};
use std::path::Path;

pub(crate) struct Images {
    params: RuntimeInitializeParams,
    engine: Option<ImageEngine>,
}

enum ImageEngine {
    Segmentation(segmentation::Segmentation),
    Upscaling(upscaling::Upscaling),
    Inpainting(inpainting::Inpainting),
    Selection(selection::Selection),
    Depth(depth::Depth),
}

impl Images {
    pub(crate) fn new(params: &RuntimeInitializeParams) -> Result<Self, String> {
        if params.device_profile.device != "cpu"
            || !(1..=8).contains(&params.device_profile.cpu_threads)
        {
            return Err("unsupported_device: images require admitted CPU threads".into());
        }
        let valid = if params.model_paths.image_segmentation.is_some() {
            params.capability
                == knovum_inference_contracts::runtime::RuntimeCapability::ImageSegmentation
        } else {
            params
                .model_paths
                .image_processing
                .as_ref()
                .is_some_and(|model| model.variant.capability() == params.capability)
        };
        if !valid {
            return Err("runtime_mismatch: image model path and capability differ".into());
        }
        Ok(Self {
            params: params.clone(),
            engine: None,
        })
    }

    fn load(&mut self) -> Result<&mut ImageEngine, String> {
        if self.engine.is_none() {
            self.engine = Some(Self::load_engine(&self.params)?);
        }
        self.engine
            .as_mut()
            .ok_or("worker_crashed: image model is unavailable".into())
    }

    fn load_engine(params: &RuntimeInitializeParams) -> Result<ImageEngine, String> {
        let threads = params.device_profile.cpu_threads;
        if let Some(path) = &params.model_paths.image_segmentation {
            return segmentation::Segmentation::new(Path::new(path), threads)
                .map(ImageEngine::Segmentation);
        }
        let model = params
            .model_paths
            .image_processing
            .as_ref()
            .ok_or("model_missing: no image model path")?;
        let directory = Path::new(&model.directory);
        use ImageModelVariant as V;
        match model.variant {
            V::RealEsrganGeneralX4v3 | V::RealEsrganX4plus => {
                upscaling::Upscaling::new(directory, model.variant == V::RealEsrganX4plus, threads)
                    .map(ImageEngine::Upscaling)
            }
            V::Migan => {
                inpainting::Inpainting::new(directory, threads).map(ImageEngine::Inpainting)
            }
            V::EfficientSamTi | V::EfficientSamS => {
                selection::Selection::new(directory, model.variant == V::EfficientSamS, threads)
                    .map(ImageEngine::Selection)
            }
            V::DepthAnythingV2SmallUint8 | V::DepthAnythingV2Small => depth::Depth::new(
                directory,
                model.variant == V::DepthAnythingV2SmallUint8,
                threads,
            )
            .map(ImageEngine::Depth),
        }
    }

    pub(crate) fn segment(
        &mut self,
        work: &Path,
        params: ImageSegmentParams,
        cancel: &Cancellation,
    ) -> Result<ImageSegmentation, String> {
        match self.load()? {
            ImageEngine::Segmentation(engine) => engine.execute(work, params, cancel),
            _ => Err("runtime_mismatch: worker does not own segmentation".into()),
        }
    }

    pub(crate) fn compute(
        &mut self,
        work: &Path,
        params: ImageComputeParams,
        cancel: &Cancellation,
    ) -> Result<ImageComputationOutput, String> {
        let [batch, channels, height, width] = params.tensor_ref.shape;
        let dtype = params.tensor_ref.data_type;
        if batch != 1 || channels != 3 {
            return Err("invalid_input: image must be one RGB tensor".into());
        }
        let input = artifact::read_tensor(work, &params.tensor_ref, 64 * 1024 * 1024)?;
        let (pixels, shape, output_dtype) = match (self.load()?, &params.computation, dtype) {
            (ImageEngine::Upscaling(engine), ImageComputation::Upscale, TensorDataType::Uint8) => (
                engine.execute(params.tensor_ref.shape, &input, cancel)?,
                [1, 3, height * 4, width * 4],
                TensorDataType::Uint8,
            ),
            (
                ImageEngine::Inpainting(engine),
                ImageComputation::Inpaint { mask_ref },
                TensorDataType::Uint8,
            ) => {
                if mask_ref.data_type != TensorDataType::Uint8
                    || mask_ref.shape != [1, 1, height, width]
                {
                    return Err("invalid_input: inpainting mask shape or type differs".into());
                }
                let mask = artifact::read_tensor(work, mask_ref, 16 * 1024 * 1024)?;
                (
                    engine.execute(params.tensor_ref.shape, &input, &mask, cancel)?,
                    params.tensor_ref.shape,
                    TensorDataType::Uint8,
                )
            }
            (
                ImageEngine::Selection(engine),
                ImageComputation::Select { points, box_rect },
                TensorDataType::Uint8,
            ) => (
                engine.execute(
                    params.tensor_ref.shape,
                    &input,
                    format!("{}:{}", params.input_revision, params.tensor_ref.sha256),
                    points,
                    box_rect.as_ref(),
                    cancel,
                )?,
                [1, 1, height, width],
                TensorDataType::Uint8,
            ),
            (
                ImageEngine::Depth(engine),
                ImageComputation::Depth,
                TensorDataType::Float32LittleEndian,
            ) => (
                engine.execute(params.tensor_ref.shape, &input, cancel)?,
                [1, 1, height, width],
                TensorDataType::Float32LittleEndian,
            ),
            _ => return Err("runtime_mismatch: image operation or input type differs".into()),
        };
        cancel.check()?;
        let tensor_ref = artifact::write_tensor(
            work,
            &format!("image-{}.tensor", params.request_id),
            shape,
            output_dtype,
            &pixels,
        )?;
        Ok(ImageComputationOutput {
            request_id: params.request_id,
            attempt_epoch: params.attempt_epoch,
            model_spec_id: params.model_spec_id,
            tensor_ref,
        })
    }
}

#[cfg(test)]
#[path = "tests/pixels.rs"]
mod tests;
