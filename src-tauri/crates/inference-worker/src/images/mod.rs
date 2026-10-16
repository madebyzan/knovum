mod depth;
mod discovery;
mod engine;
mod inpainting;
mod prompts;
mod segmentation;
mod selection;
mod session;
mod upscaling;

use crate::{artifact, cancellation::Cancellation};
use engine::ImageEngine;
use knovum_inference_contracts::{
    image_compute::{ImageComputationOutput, ImageComputeParams},
    image_wire::{ImageSegmentParams, ImageSegmentation},
    runtime::RuntimeInitializeParams,
};
use std::path::Path;

pub(crate) struct Images {
    params: RuntimeInitializeParams,
    engine: Option<ImageEngine>,
}

impl Images {
    pub(crate) fn new(params: &RuntimeInitializeParams) -> Result<Self, String> {
        if params.device_profile.device != "cpu"
            || !(1..=8).contains(&params.device_profile.cpu_threads)
        {
            return Err("unsupported_device: images require admitted CPU threads".into());
        }
        if ImageEngine::capability(params) != Some(params.capability) {
            return Err("runtime_mismatch: image model path and capability differ".into());
        }
        Ok(Self {
            params: params.clone(),
            engine: None,
        })
    }

    fn load(&mut self) -> Result<&mut ImageEngine, String> {
        if self.engine.is_none() {
            self.engine = Some(ImageEngine::load(&self.params)?);
        }
        self.engine
            .as_mut()
            .ok_or("worker_crashed: image model is unavailable".into())
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
        let [batch, channels, _, _] = params.tensor_ref.shape;
        if batch != 1 || channels != 3 {
            return Err("invalid_input: image must be one RGB tensor".into());
        }
        let input = artifact::read_tensor(work, &params.tensor_ref, 64 * 1024 * 1024)?;
        let computed = self.load()?.compute(work, &params, &input, cancel)?;
        cancel.check()?;
        let tensor_ref = artifact::write_tensor(
            work,
            &format!("image-{}.tensor", params.request_id),
            computed.shape,
            computed.data_type,
            &computed.pixels,
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
