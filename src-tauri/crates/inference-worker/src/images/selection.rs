#[cfg(test)]
#[path = "tests/selection.rs"]
mod tests;
use super::session;
use crate::cancellation::Cancellation;
use knovum_inference_contracts::image_compute::{ImageSelectionPoint, ImageSelectionRect};
use ort::{
    session::Session,
    value::{Tensor, TensorRef},
};
use std::path::Path;

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
        let [batch, channels, height, width] = shape;
        if batch != 1 || channels != 3 || u64::from(height) * u64::from(width) > 4_194_304 {
            return Err("invalid_input: selection source exceeds its budget".into());
        }
        let (coords, labels) = prompt(points, rect, height, width)?;
        cancellation.check()?;
        if !self
            .encoding
            .as_ref()
            .is_some_and(|(cached, dimensions, _)| cached == &identity && dimensions == &shape)
        {
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
            if actual.as_ref() != [1, 256, 64, 64] || values.iter().any(|value| !value.is_finite())
            {
                return Err("invalid_output: selection encoding differs".into());
            }
            self.encoding = Some((identity, shape, values.to_vec()));
        }
        let (_, _, encoding) = self
            .encoding
            .as_ref()
            .ok_or("invalid_output: selection encoding unavailable")?;
        let tensor = TensorRef::from_array_view(([1usize, 256, 64, 64], encoding.as_slice()))
            .map_err(|error| format!("invalid_input: {error}"))?;
        let count = labels.len();
        let coords = Tensor::from_array(([1usize, 1, count, 2], coords))
            .map_err(|error| format!("invalid_input: {error}"))?;
        let labels = Tensor::from_array(([1usize, 1, count], labels))
            .map_err(|error| format!("invalid_input: {error}"))?;
        let size = Tensor::from_array(([2usize], vec![i64::from(height), i64::from(width)]))
            .map_err(|error| format!("invalid_input: {error}"))?;
        let options = cancellation.run_options()?;
        let decoded = self.decoder.run_with_options(ort::inputs!["image_embeddings" => tensor,"batched_point_coords" => coords,"batched_point_labels" => labels,"orig_im_size" => size],&options).map_err(|error| format!("invalid_output: {error}"))?;
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
        let pixels = (height as usize) * (width as usize);
        Ok(masks[best * pixels..(best + 1) * pixels]
            .iter()
            .map(|value| if *value > 0.0 { 255 } else { 0 })
            .collect())
    }
}

pub(super) fn prompt(
    points: &[ImageSelectionPoint],
    rect: Option<&ImageSelectionRect>,
    height: u32,
    width: u32,
) -> Result<(Vec<f32>, Vec<f32>), String> {
    let count = points.len() + usize::from(rect.is_some()) * 2;
    if !(1..=6).contains(&count) || rect.is_none() && !points.iter().any(|point| point.foreground) {
        return Err(
            "invalid_input: selection needs one to six prompts and a foreground point or box"
                .into(),
        );
    }
    let mut coords = Vec::with_capacity(count * 2);
    let mut labels = Vec::with_capacity(count);
    for point in points {
        if !point.x.is_finite()
            || !point.y.is_finite()
            || !(0.0..=1.0).contains(&point.x)
            || !(0.0..=1.0).contains(&point.y)
        {
            return Err("invalid_input: selection point is outside image".into());
        }
        coords.extend([point.x * width as f32, point.y * height as f32]);
        labels.push(if point.foreground { 1.0 } else { 0.0 });
    }
    if let Some(rect) = rect {
        if [rect.x, rect.y, rect.width, rect.height]
            .iter()
            .any(|value| !value.is_finite())
            || rect.x < 0.0
            || rect.y < 0.0
            || rect.width <= 0.0
            || rect.height <= 0.0
            || rect.x + rect.width > 1.0
            || rect.y + rect.height > 1.0
        {
            return Err("invalid_input: selection box is outside image".into());
        }
        coords.extend([
            rect.x * width as f32,
            rect.y * height as f32,
            (rect.x + rect.width) * width as f32,
            (rect.y + rect.height) * height as f32,
        ]);
        labels.extend([2.0, 3.0]);
    }
    Ok((coords, labels))
}
