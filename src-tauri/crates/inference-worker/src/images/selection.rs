use super::{prompts, session};
use crate::cancellation::Cancellation;
use knovum_inference_contracts::image_compute::{ImageSelectionPoint, ImageSelectionRect};
use ort::{
    session::Session,
    value::{Tensor, TensorRef},
};
use std::path::Path;

/// EfficientSAM always returns three candidate masks per prompt together with their predicted
/// IoU. Decoded logits are kept unthresholded so callers that need a stability score can
/// re-threshold the same pass.
pub(super) struct DecodedPrompt {
    logits: Vec<f32>,
    best: usize,
    pub(super) score: f32,
}

impl DecodedPrompt {
    /// Thresholds the highest scoring mask; logits above `threshold` are inside it.
    pub(super) fn mask(&self, threshold: f32) -> impl Iterator<Item = bool> + '_ {
        let pixels = self.logits.len() / 3;
        self.logits[self.best * pixels..(self.best + 1) * pixels]
            .iter()
            .map(move |value| *value > threshold)
    }
}

pub(super) struct Selection {
    encoder: Session,
    decoder: Session,
    encoding: Option<(String, [u32; 4], Vec<f32>)>,
}

impl Selection {
    pub(super) fn new(directory: &Path, quality: bool, threads: u8) -> Result<Self, String> {
        let name = if quality { "s" } else { "ti" };
        Ok(Self {
            encoder: session::load(
                &directory.join(format!("efficientsam_{name}_encoder.onnx")),
                threads,
                &["batched_images"],
                &["image_embeddings"],
            )?,
            decoder: session::load(
                &directory.join(format!("efficientsam_{name}_decoder.onnx")),
                threads,
                &[
                    "image_embeddings",
                    "batched_point_coords",
                    "batched_point_labels",
                    "orig_im_size",
                ],
                &["output_masks", "iou_predictions"],
            )?,
            encoding: None,
        })
    }

    pub(super) fn execute(
        &mut self,
        shape: [u32; 4],
        image: &[u8],
        identity: String,
        points: &[ImageSelectionPoint],
        rect: Option<&ImageSelectionRect>,
        cancellation: &Cancellation,
    ) -> Result<Vec<u8>, String> {
        let (height, width) = source_size(shape)?;
        let (coords, labels) = prompts::build(points, rect, height, width)?;
        cancellation.check()?;
        self.encode(shape, image, identity, cancellation)?;
        let decoded = self.decode(coords, labels, (height, width), cancellation)?;
        Ok(decoded
            .mask(0.0)
            .map(|inside| if inside { 255 } else { 0 })
            .collect())
    }

    /// Encodes the source unless the cached embedding already belongs to it. The embedding is the
    /// expensive half of EfficientSAM, so every prompt over one source reuses this pass.
    pub(super) fn encode(
        &mut self,
        shape: [u32; 4],
        image: &[u8],
        identity: String,
        cancellation: &Cancellation,
    ) -> Result<(), String> {
        if self
            .encoding
            .as_ref()
            .is_some_and(|(cached, dimensions, _)| cached == &identity && dimensions == &shape)
        {
            return Ok(());
        }
        let tensor = Tensor::from_array((
            shape.map(|value| value as usize),
            image
                .iter()
                .map(|value| f32::from(*value) / 255.0)
                .collect::<Vec<_>>(),
        ))
        .map_err(|error| format!("invalid_input: {error}"))?;
        let options = cancellation.run_options()?;
        let encoded = self
            .encoder
            .run_with_options(ort::inputs!["batched_images" => tensor], &options)
            .map_err(|error| format!("invalid_output: {error}"))?;
        cancellation.check()?;
        let (actual, values) = encoded["image_embeddings"]
            .try_extract_tensor::<f32>()
            .map_err(|error| format!("invalid_output: {error}"))?;
        if actual.as_ref() != [1, 256, 64, 64] || values.iter().any(|value| !value.is_finite()) {
            return Err("invalid_output: selection encoding differs".into());
        }
        self.encoding = Some((identity, shape, values.to_vec()));
        Ok(())
    }

    /// Decodes one prompt at the requested mask resolution. `orig_im_size` is independent of the
    /// encoded source, so discovery can decode cheap small masks before refining the survivors.
    pub(super) fn decode(
        &mut self,
        coords: Vec<f32>,
        labels: Vec<f32>,
        (height, width): (u32, u32),
        cancellation: &Cancellation,
    ) -> Result<DecodedPrompt, String> {
        let (_, _, encoding) = self
            .encoding
            .as_ref()
            .ok_or("invalid_output: selection encoding unavailable")?;
        let embeddings = TensorRef::from_array_view(([1usize, 256, 64, 64], encoding.as_slice()))
            .map_err(|error| format!("invalid_input: {error}"))?;
        let count = labels.len();
        let coords = Tensor::from_array(([1usize, 1, count, 2], coords))
            .map_err(|error| format!("invalid_input: {error}"))?;
        let labels = Tensor::from_array(([1usize, 1, count], labels))
            .map_err(|error| format!("invalid_input: {error}"))?;
        let size = Tensor::from_array(([2usize], vec![i64::from(height), i64::from(width)]))
            .map_err(|error| format!("invalid_input: {error}"))?;
        let options = cancellation.run_options()?;
        let decoded = self.decoder.run_with_options(ort::inputs!["image_embeddings" => embeddings,"batched_point_coords" => coords,"batched_point_labels" => labels,"orig_im_size" => size],&options).map_err(|error| format!("invalid_output: {error}"))?;
        cancellation.check()?;
        let (mask_shape, masks) = decoded["output_masks"]
            .try_extract_tensor::<f32>()
            .map_err(|error| format!("invalid_output: {error}"))?;
        let (score_shape, scores) = decoded["iou_predictions"]
            .try_extract_tensor::<f32>()
            .map_err(|error| format!("invalid_output: {error}"))?;
        if mask_shape.as_ref() != [1, 1, 3, i64::from(height), i64::from(width)]
            || score_shape.as_ref() != [1, 1, 3]
            || masks.iter().chain(scores).any(|value| !value.is_finite())
        {
            return Err("invalid_output: selection output differs".into());
        }
        let best = (1..3).fold(0, |best, index| {
            if scores[index] > scores[best] {
                index
            } else {
                best
            }
        });
        Ok(DecodedPrompt {
            logits: masks.to_vec(),
            best,
            score: scores[best],
        })
    }
}

pub(super) fn source_size(shape: [u32; 4]) -> Result<(u32, u32), String> {
    let [batch, channels, height, width] = shape;
    if batch != 1 || channels != 3 || u64::from(height) * u64::from(width) > 4_194_304 {
        return Err("invalid_input: selection source exceeds its budget".into());
    }
    Ok((height, width))
}
