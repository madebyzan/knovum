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
    runtime::{RuntimeCapability, RuntimeInitializeParams, RuntimeModel},
};
use std::path::Path;

pub(crate) struct Images(ImageEngine);

/// Admits the image execution profile. Pure in its initialization message, so an unadmitted thread
/// count or a model that does not implement the frozen capability is rejected before a graph loads.
fn admit(params: &RuntimeInitializeParams) -> Result<(), String> {
    if params.device_profile.device != "cpu"
        || !(1..=8).contains(&params.device_profile.cpu_threads)
    {
        return Err("unsupported_device: images require admitted CPU threads".into());
    }
    let implemented = match &params.model {
        RuntimeModel::Snapshot { .. } => RuntimeCapability::ImageSegmentation,
        RuntimeModel::ImageVariant { variant, .. } => variant.capability(),
    };
    if implemented != params.capability {
        return Err("runtime_mismatch: image model and capability differ".into());
    }
    Ok(())
}

impl Images {
    pub(crate) fn new(params: &RuntimeInitializeParams) -> Result<Self, String> {
        admit(params)?;
        ImageEngine::load(params).map(Self)
    }

    pub(crate) fn segment(
        &mut self,
        work: &Path,
        params: ImageSegmentParams,
        cancel: &Cancellation,
    ) -> Result<ImageSegmentation, String> {
        match &mut self.0 {
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
        let computed = self.0.compute(work, &params, &input, cancel)?;
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
mod tests;
