use std::path::Path;

use knovum_inference_contracts::image_wire::{
    ImageSegmentParams, ImageSegmentation, TensorDataType,
};
use ort::{session::Session, value::Tensor};

use super::{scalar_field, session};
use crate::{artifact, cancellation::Cancellation};

pub(super) struct Segmentation {
    session: Session,
    input_name: String,
    output_name: String,
}

impl Segmentation {
    pub(super) fn new(path: &Path, threads: u8) -> Result<Self, String> {
        let session = (|| -> ort::Result<Session> {
            Session::builder()?
                .with_execution_providers([ort::ep::CPU::default()
                    .with_arena_allocator(true)
                    .build()
                    .error_on_failure()])?
                .with_intra_threads(usize::from(threads))?
                .with_inter_threads(1)?
                .commit_from_file(path)
        })()
        .map_err(|error| format!("model_corrupt: {error}"))?;
        if session.inputs().len() != 1 || session.outputs().is_empty() {
            return Err("model_corrupt: segmentation graph interface differs".into());
        }
        let input_name = session.inputs()[0].name().into();
        let output_name = session.outputs()[0].name().into();
        Ok(Self {
            session,
            input_name,
            output_name,
        })
    }

    pub(super) fn execute(
        &mut self,
        work: &Path,
        params: ImageSegmentParams,
        cancellation: &Cancellation,
    ) -> Result<ImageSegmentation, String> {
        let shape = params.tensor_ref.shape;
        if shape[0] != 1
            || shape[1] != 3
            || shape[2] != shape[3]
            || params.tensor_ref.data_type != TensorDataType::Float32LittleEndian
        {
            return Err("invalid_input: segmentation requires a square float RGB tensor".into());
        }
        let edge = shape[2];
        let input_bytes = 3 * u64::from(edge) * u64::from(edge) * 4;
        let bytes = artifact::read_tensor(work, &params.tensor_ref, input_bytes)?;
        let tensor =
            Tensor::from_array((shape.map(|value| value as usize), session::floats(&bytes)))
                .map_err(|error| format!("invalid_input: {error}"))?;
        let options = cancellation.run_options()?;
        let outputs = self
            .session
            .run_with_options(ort::inputs![self.input_name.as_str() => tensor], &options)
            .map_err(|error| format!("invalid_output: {error}"))?;
        cancellation.check()?;
        let (out_shape, values) = outputs[self.output_name.as_str()]
            .try_extract_tensor::<f32>()
            .map_err(|error| format!("invalid_output: {error}"))?;
        let expected_mask_shape: [u32; 4] = [1, 1, edge, edge];
        if out_shape.as_ref() != expected_mask_shape.map(i64::from) {
            return Err("invalid_output: segmentation mask shape differs".into());
        }
        // Reference post-processing: the first head's field stretched to the full [0, 1] range.
        let field = scalar_field::normalize(values)?;
        let mask_ref = artifact::write_tensor(
            work,
            &format!("mask-{}.f32", params.request_id),
            expected_mask_shape,
            TensorDataType::Float32LittleEndian,
            &session::float_bytes(&field)?,
        )?;
        Ok(ImageSegmentation {
            request_id: params.request_id,
            attempt_epoch: params.attempt_epoch,
            model_spec_id: params.model_spec_id,
            mask_ref,
        })
    }
}
