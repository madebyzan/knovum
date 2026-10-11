use super::session;
use crate::cancellation::Cancellation;
use ort::{session::Session, value::Tensor};
use std::path::Path;

pub(super) struct Inpainting {
    session: Session,
}

impl Inpainting {
    pub(super) fn new(directory: &Path, threads: u8) -> Result<Self, String> {
        Ok(Self {
            session: session::load(
                &directory.join("migan_pipeline_v2.onnx"),
                threads,
                &["image", "mask"],
                &["result"],
            )?,
        })
    }

    pub(super) fn execute(
        &mut self,
        shape: [u32; 4],
        image: &[u8],
        mask: &[u8],
        cancellation: &Cancellation,
    ) -> Result<Vec<u8>, String> {
        let [batch, channels, height, width] = shape;
        if batch != 1
            || channels != 3
            || u64::from(height) * u64::from(width) > 16_777_216
            || image.len() != mask.len() * 3
        {
            return Err("invalid_input: inpainting tensor shapes differ".into());
        }
        let input = Tensor::from_array((shape.map(|value| value as usize), image.to_vec()))
            .map_err(|error| format!("invalid_input: {error}"))?;
        let known = Tensor::from_array((
            [1usize, 1, height as usize, width as usize],
            known_mask(mask)?,
        ))
        .map_err(|error| format!("invalid_input: {error}"))?;
        let options = cancellation.run_options()?;
        let outputs = self
            .session
            .run_with_options(ort::inputs!["image" => input, "mask" => known], &options)
            .map_err(|error| format!("invalid_output: {error}"))?;
        cancellation.check()?;
        let (actual, pixels) = outputs["result"]
            .try_extract_tensor::<u8>()
            .map_err(|error| format!("invalid_output: {error}"))?;
        if actual.as_ref() != shape.map(i64::from) {
            return Err("invalid_output: inpainting output shape differs".into());
        }
        composite(image, pixels, mask)
    }
}

pub(super) fn known_mask(mask: &[u8]) -> Result<Vec<u8>, String> {
    if !mask.contains(&255) || mask.iter().any(|value| !matches!(value, 0 | 255)) {
        return Err("invalid_input: inpainting requires a nonempty binary mask".into());
    }
    Ok(mask.iter().map(|value| 255 - value).collect())
}

pub(super) fn composite(source: &[u8], output: &[u8], mask: &[u8]) -> Result<Vec<u8>, String> {
    if mask.is_empty() || source.len() != output.len() || source.len() != mask.len() * 3 {
        return Err("invalid_output: inpainting compositor shapes differ".into());
    }
    Ok(source
        .iter()
        .zip(output)
        .enumerate()
        .map(|(index, (original, painted))| {
            if mask[index % mask.len()] == 255 {
                *painted
            } else {
                *original
            }
        })
        .collect())
}
