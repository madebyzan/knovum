use super::session;
use crate::cancellation::Cancellation;
use ort::{session::Session, value::Tensor};
use std::path::Path;

pub(super) struct Depth {
    session: Session,
}

impl Depth {
    pub(super) fn new(directory: &Path, quantized: bool, threads: u8) -> Result<Self, String> {
        let name = if quantized {
            "model_uint8.onnx"
        } else {
            "model.onnx"
        };
        Ok(Self {
            session: session::load(
                &directory.join(name),
                threads,
                &["pixel_values"],
                &["predicted_depth"],
            )?,
        })
    }

    pub(super) fn execute(
        &mut self,
        shape: [u32; 4],
        bytes: &[u8],
        cancellation: &Cancellation,
    ) -> Result<Vec<u8>, String> {
        let [batch, channels, height, width] = shape;
        if batch != 1
            || channels != 3
            || height % 14 != 0
            || width % 14 != 0
            || u64::from(height) * u64::from(width) > 1_048_576
        {
            return Err("invalid_input: depth tensor exceeds its processor budget".into());
        }
        let input = Tensor::from_array((shape.map(|value| value as usize), session::floats(bytes)))
            .map_err(|error| format!("invalid_input: {error}"))?;
        let options = cancellation.run_options()?;
        let outputs = self
            .session
            .run_with_options(ort::inputs!["pixel_values" => input], &options)
            .map_err(|error| format!("invalid_output: {error}"))?;
        cancellation.check()?;
        let (actual, values) = outputs["predicted_depth"]
            .try_extract_tensor::<f32>()
            .map_err(|error| format!("invalid_output: {error}"))?;
        if actual.as_ref() != [1, i64::from(height), i64::from(width)] {
            return Err("invalid_output: depth shape differs".into());
        }
        session::float_bytes(&normalize(values)?)
    }
}

pub(super) fn normalize(values: &[f32]) -> Result<Vec<f32>, String> {
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err("invalid_output: invalid depth field".into());
    }
    let low = values.iter().copied().fold(f32::INFINITY, f32::min);
    let high = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    if high <= low || !(high - low).is_finite() {
        return Err("invalid_output: constant or overflowing depth field".into());
    }
    Ok(values
        .iter()
        .map(|value| (value - low) / (high - low))
        .collect())
}
