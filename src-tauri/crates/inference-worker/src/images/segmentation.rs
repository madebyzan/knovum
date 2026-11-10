use std::path::Path;

use knovum_inference_contracts::image_wire::{
    ImageSegmentParams, ImageSegmentation, TensorDataType, IMAGE_SEGMENTATION_INPUT_BYTES,
    IMAGE_SEGMENTATION_RESOLUTION,
};
use ort::{session::Session, value::Tensor};

use super::{scalar_field, session};

const EDGE: u32 = IMAGE_SEGMENTATION_RESOLUTION;
const INPUT_SHAPE: [u32; 4] = [1, 3, EDGE, EDGE];
const MASK_SHAPE: [u32; 4] = [1, 1, EDGE, EDGE];
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
        if params.tensor_ref.shape != INPUT_SHAPE
            || params.tensor_ref.data_type != TensorDataType::Float32LittleEndian
        {
            return Err("invalid_input: segmentation requires a 320x320 float RGB tensor".into());
        }
        let bytes =
            artifact::read_tensor(work, &params.tensor_ref, IMAGE_SEGMENTATION_INPUT_BYTES)?;
        let tensor = Tensor::from_array((
            INPUT_SHAPE.map(|value| value as usize),
            session::floats(&bytes),
        ))
        .map_err(|error| format!("invalid_input: {error}"))?;
        let options = cancellation.run_options()?;
        let outputs = self
            .session
            .run_with_options(ort::inputs![self.input_name.as_str() => tensor], &options)
            .map_err(|error| format!("invalid_output: {error}"))?;
        cancellation.check()?;
        let (shape, values) = outputs[self.output_name.as_str()]
            .try_extract_tensor::<f32>()
            .map_err(|error| format!("invalid_output: {error}"))?;
        if shape.as_ref() != MASK_SHAPE.map(i64::from) {
            return Err("invalid_output: segmentation mask shape differs".into());
        }
        // Reference post-processing: the first head's field stretched to the full [0, 1] range.
        let field = scalar_field::normalize(values)?;
        let mask_ref = artifact::write_tensor(
            work,
            &format!("mask-{}.f32", params.request_id),
            MASK_SHAPE,
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
