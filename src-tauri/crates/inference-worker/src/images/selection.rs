use super::{prompts, session};
use crate::cancellation::Cancellation;
use image::{
    imageops::{self, FilterType},
    Rgb, RgbImage,
};
use knovum_inference_contracts::{
    image_compute::{ImageSelectionPoint, ImageSelectionRect},
    image_spec::{Geometry, Standardization, MOBILE_SAM_SPEC, SAM2_HIERA_TINY_SPEC},
};
use ort::{
    session::{RunOptions, Session, SessionOutputs},
    value::{Tensor, TensorRef},
};
use std::path::Path;

/// MobileSAM resizes the longest edge and bakes padding and normalization into its encoder.
const SAM1_EDGE: u32 = match MOBILE_SAM_SPEC.geometry {
    Geometry::LongestEdge { edge } => edge,
    _ => panic!("frozen MobileSAM geometry"),
};
/// SAM 2.1 stretches the source into a fixed square and standardizes caller-side.
const SAM2_WIDTH: u32 = match SAM2_HIERA_TINY_SPEC.geometry {
    Geometry::Stretch { width, .. } => width,
    _ => panic!("frozen SAM 2.1 geometry"),
};
const SAM2_HEIGHT: u32 = match SAM2_HIERA_TINY_SPEC.geometry {
    Geometry::Stretch { height, .. } => height,
    _ => panic!("frozen SAM 2.1 geometry"),
};
const SAM2_MEAN: [f32; 3] = match SAM2_HIERA_TINY_SPEC.standardization {
    Standardization::ImageNet { mean, .. } => mean,
    _ => panic!("frozen SAM 2.1 standardization"),
};
const SAM2_STD: [f32; 3] = match SAM2_HIERA_TINY_SPEC.standardization {
    Standardization::ImageNet { std, .. } => std,
    _ => panic!("frozen SAM 2.1 standardization"),
};
/// Both families score masks over a 256² logits grid.
const MASK_GRID: usize = 256;
const GRID_PIXELS: usize = MASK_GRID * MASK_GRID;
/// Frozen embedding shapes, in decoder input order per family.
const SAM1_EMBEDDINGS: [&[i64]; 1] = [&[1, 256, 64, 64]];
const SAM2_EMBEDDINGS: [&[i64]; 3] = [&[1, 256, 64, 64], &[1, 32, 256, 256], &[1, 64, 128, 128]];

/// One decoded prompt: the mask logits at the requested resolution and the decoder's IoU score.
/// Logits are kept unthresholded so callers that need a stability score can re-threshold the same
/// pass.
pub(super) struct DecodedPrompt {
    logits: Vec<f32>,
    pub(super) score: f32,
}

impl DecodedPrompt {
    /// Thresholds the mask; logits above `threshold` are inside it.
    pub(super) fn mask(&self, threshold: f32) -> impl Iterator<Item = bool> + '_ {
        self.logits.iter().map(move |value| *value > threshold)
    }
}

/// Which frozen decoder contract the loaded pair speaks.
#[derive(Clone, Copy)]
enum Family {
    /// MobileSAM: SAM 1's decoder, with a padding prompt point and low-resolution postprocess.
    Sam1,
    /// SAM 2.1: three scored candidate masks over a stretched square.
    Sam2,
}

pub(super) struct Selection {
    family: Family,
    encoder: Session,
    decoder: Session,
    /// Identity, source shape, and the embeddings in decoder input order.
    encoding: Option<(String, [u32; 4], Vec<Vec<f32>>)>,
}

impl Selection {
    pub(super) fn new(directory: &Path, quality: bool, threads: u8) -> Result<Self, String> {
        if quality {
            Ok(Self {
                family: Family::Sam2,
                encoder: session::load(
                    &directory.join("sam2.1_hiera_tiny.encoder.onnx"),
                    threads,
                    &["image"],
                    &["image_embed", "high_res_feats_0", "high_res_feats_1"],
                )?,
                decoder: session::load(
                    &directory.join("sam2.1_hiera_tiny.decoder.onnx"),
                    threads,
                    &[
                        "image_embed",
                        "high_res_feats_0",
                        "high_res_feats_1",
                        "point_coords",
                        "point_labels",
                        "mask_input",
                        "has_mask_input",
                    ],
                    &["masks", "iou_predictions"],
                )?,
                encoding: None,
            })
        } else {
            Ok(Self {
                family: Family::Sam1,
                encoder: session::load(
                    &directory.join("mobile_sam.encoder.onnx"),
                    threads,
                    &["input_image"],
                    &["image_embeddings"],
                )?,
                decoder: session::load(
                    &directory.join("sam_vit_h_4b8939.decoder.onnx"),
                    threads,
                    &[
                        "image_embeddings",
                        "point_coords",
                        "point_labels",
                        "mask_input",
                        "has_mask_input",
                        "orig_im_size",
                    ],
                    &["low_res_masks", "iou_predictions"],
                )?,
                encoding: None,
            })
        }
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

    /// Encodes the source unless the cached embeddings already belong to it. The encoder is the
    /// expensive half of either family, so every prompt over one source reuses this pass.
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
        let options = cancellation.run_options()?;
        let embeddings = match self.family {
            Family::Sam1 => encode_sam1(&mut self.encoder, shape, image, &options),
            Family::Sam2 => encode_sam2(&mut self.encoder, shape, image, &options),
        }?;
        cancellation.check()?;
        self.encoding = Some((identity, shape, embeddings));
        Ok(())
    }

    /// Decodes one prompt into a mask at the requested resolution. Prompt coordinates are in
    /// source pixels; each family owns its own transform into model space. The target resolution
    /// is independent of the encoded source, so discovery can decode cheap small masks before
    /// refining the survivors.
    pub(super) fn decode(
        &mut self,
        coords: Vec<f32>,
        labels: Vec<f32>,
        (height, width): (u32, u32),
        cancellation: &Cancellation,
    ) -> Result<DecodedPrompt, String> {
        let (_, shape, embeddings) = self
            .encoding
            .as_ref()
            .ok_or("invalid_output: selection encoding unavailable")?;
        let [_, _, source_height, source_width] = *shape;
        let source = (source_height, source_width);
        let target = (height, width);
        let options = cancellation.run_options()?;
        let decoded = match self.family {
            Family::Sam1 => decode_sam1(
                &mut self.decoder,
                embeddings,
                coords,
                labels,
                source,
                target,
                &options,
            ),
            Family::Sam2 => decode_sam2(
                &mut self.decoder,
                embeddings,
                coords,
                labels,
                source,
                target,
                &options,
            ),
        }?;
        cancellation.check()?;
        Ok(decoded)
    }
}

/// The MobileSAM encoder reads raw 0-255 pixels in the resized longest-edge frame and pads and
/// normalizes them itself.
fn encode_sam1(
    encoder: &mut Session,
    shape: [u32; 4],
    image: &[u8],
    options: &RunOptions,
) -> Result<Vec<Vec<f32>>, String> {
    let [_, _, height, width] = shape;
    let (new_height, new_width) = preprocess_shape(height, width);
    let resized = imageops::resize(
        &rgb_image(shape, image),
        new_width,
        new_height,
        FilterType::Triangle,
    );
    let tensor = Tensor::from_array((
        [new_height as usize, new_width as usize, 3],
        resized
            .into_raw()
            .into_iter()
            .map(f32::from)
            .collect::<Vec<_>>(),
    ))
    .map_err(|error| format!("invalid_input: {error}"))?;
    let encoded = encoder
        .run_with_options(ort::inputs!["input_image" => tensor], options)
        .map_err(|error| format!("invalid_output: {error}"))?;
    extract_embeddings(&encoded, &["image_embeddings"], &SAM1_EMBEDDINGS)
}

/// The SAM 2.1 encoder reads a stretched, unit-scaled, ImageNet-standardized square.
fn encode_sam2(
    encoder: &mut Session,
    shape: [u32; 4],
    image: &[u8],
    options: &RunOptions,
) -> Result<Vec<Vec<f32>>, String> {
    let resized = imageops::resize(
        &rgb_image(shape, image),
        SAM2_WIDTH,
        SAM2_HEIGHT,
        FilterType::Triangle,
    );
    let pixels = resized.as_raw();
    let tensor = Tensor::from_array((
        [1usize, 3, SAM2_HEIGHT as usize, SAM2_WIDTH as usize],
        (0..3)
            .flat_map(|channel| {
                pixels.chunks_exact(3).map(move |pixel| {
                    (f32::from(pixel[channel]) / 255.0 - SAM2_MEAN[channel]) / SAM2_STD[channel]
                })
            })
            .collect::<Vec<_>>(),
    ))
    .map_err(|error| format!("invalid_input: {error}"))?;
    let encoded = encoder
        .run_with_options(ort::inputs!["image" => tensor], options)
        .map_err(|error| format!("invalid_output: {error}"))?;
    extract_embeddings(
        &encoded,
        &["image_embed", "high_res_feats_0", "high_res_feats_1"],
        &SAM2_EMBEDDINGS,
    )
}

/// SAM 1's export froze the aspect ratio of its high-resolution output, so the low-resolution
/// logits get the reference postprocess instead: bilinear upscale to the padded square, crop away
/// the padding, then resample to the requested resolution.
fn decode_sam1(
    decoder: &mut Session,
    embeddings: &[Vec<f32>],
    coords: Vec<f32>,
    labels: Vec<f32>,
    source: (u32, u32),
    target: (u32, u32),
    options: &RunOptions,
) -> Result<DecodedPrompt, String> {
    let (coords, labels) = sam1_prompt(coords, labels, source);
    let count = labels.len();
    let embeddings = embedding(&embeddings[0], SAM1_EMBEDDINGS[0])?;
    let coords = tensor([1usize, count, 2], coords)?;
    let labels = tensor([1usize, count], labels)?;
    let mask_input = tensor(
        [1usize, 1, MASK_GRID, MASK_GRID],
        vec![0.0_f32; GRID_PIXELS],
    )?;
    let has_mask_input = tensor([1usize], vec![0.0_f32])?;
    let orig_im_size = tensor([2usize], vec![source.0 as f32, source.1 as f32])?;
    let decoded = decoder
        .run_with_options(
            ort::inputs![
                "image_embeddings" => embeddings,
                "point_coords" => coords,
                "point_labels" => labels,
                "mask_input" => mask_input,
                "has_mask_input" => has_mask_input,
                "orig_im_size" => orig_im_size,
            ],
            options,
        )
        .map_err(|error| format!("invalid_output: {error}"))?;
    let (masks, scores) = extract_masks(&decoded, "low_res_masks", [1, 1], 1)?;
    let logits = sam1_postprocess(masks, preprocess_shape(source.0, source.1), target);
    Ok(DecodedPrompt {
        logits,
        score: scores[0],
    })
}

/// SAM 2.1 scores three candidate masks over the 256² grid; the strongest becomes the prompt.
fn decode_sam2(
    decoder: &mut Session,
    embeddings: &[Vec<f32>],
    coords: Vec<f32>,
    labels: Vec<f32>,
    source: (u32, u32),
    target: (u32, u32),
    options: &RunOptions,
) -> Result<DecodedPrompt, String> {
    let coords = sam2_coords(coords, source);
    let count = labels.len();
    let image_embed = embedding(&embeddings[0], SAM2_EMBEDDINGS[0])?;
    let feats0 = embedding(&embeddings[1], SAM2_EMBEDDINGS[1])?;
    let feats1 = embedding(&embeddings[2], SAM2_EMBEDDINGS[2])?;
    let coords = tensor([1usize, count, 2], coords)?;
    let labels = tensor([1usize, count], labels)?;
    let mask_input = tensor(
        [1usize, 1, MASK_GRID, MASK_GRID],
        vec![0.0_f32; GRID_PIXELS],
    )?;
    let has_mask_input = tensor([1usize], vec![0.0_f32])?;
    let decoded = decoder
        .run_with_options(
            ort::inputs![
                "image_embed" => image_embed,
                "high_res_feats_0" => feats0,
                "high_res_feats_1" => feats1,
                "point_coords" => coords,
                "point_labels" => labels,
                "mask_input" => mask_input,
                "has_mask_input" => has_mask_input,
            ],
            options,
        )
        .map_err(|error| format!("invalid_output: {error}"))?;
    let (masks, scores) = extract_masks(&decoded, "masks", [1, 3], 3)?;
    let best = (1..3).fold(0, |best, index| {
        if scores[index] > scores[best] {
            index
        } else {
            best
        }
    });
    let logits = resize_logits(
        &masks[best * GRID_PIXELS..(best + 1) * GRID_PIXELS],
        (MASK_GRID as u32, MASK_GRID as u32),
        target,
    );
    Ok(DecodedPrompt {
        logits,
        score: scores[best],
    })
}

/// SAM 1's longest-edge resize, rounded exactly like the reference implementation.
fn preprocess_shape(height: u32, width: u32) -> (u32, u32) {
    let scale = f64::from(SAM1_EDGE) / f64::from(height.max(width));
    (
        (f64::from(height) * scale + 0.5) as u32,
        (f64::from(width) * scale + 0.5) as u32,
    )
}

/// Scales source-pixel prompt coordinates into SAM 1's resized frame and appends the padding
/// point the traced decoder requires: coordinate `(0, 0)` with the ignore label `-1`.
fn sam1_prompt(
    coords: Vec<f32>,
    mut labels: Vec<f32>,
    (height, width): (u32, u32),
) -> (Vec<f32>, Vec<f32>) {
    let (new_height, new_width) = preprocess_shape(height, width);
    let (x_scale, y_scale) = (
        new_width as f32 / width as f32,
        new_height as f32 / height as f32,
    );
    let mut scaled = Vec::with_capacity(coords.len() + 2);
    for (index, value) in coords.iter().enumerate() {
        scaled.push(value * if index % 2 == 0 { x_scale } else { y_scale });
    }
    scaled.extend([0.0, 0.0]);
    labels.push(-1.0);
    (scaled, labels)
}

/// Scales source-pixel prompt coordinates into SAM 2.1's stretched square.
fn sam2_coords(coords: Vec<f32>, (height, width): (u32, u32)) -> Vec<f32> {
    let (x_scale, y_scale) = (
        SAM2_WIDTH as f32 / width as f32,
        SAM2_HEIGHT as f32 / height as f32,
    );
    coords
        .iter()
        .enumerate()
        .map(|(index, value)| value * if index % 2 == 0 { x_scale } else { y_scale })
        .collect()
}

/// SAM 1's export froze the aspect ratio of its high-resolution output, so the low-resolution
/// logits get the reference postprocess instead: bilinear upscale to the padded square, crop away
/// the bottom-right padding, then resample to the requested resolution.
fn sam1_postprocess(masks: &[f32], resized: (u32, u32), target: (u32, u32)) -> Vec<f32> {
    let upscaled = resize_logits(
        masks,
        (MASK_GRID as u32, MASK_GRID as u32),
        (SAM1_EDGE, SAM1_EDGE),
    );
    let stride = SAM1_EDGE as usize;
    let cropped = upscaled
        .chunks(stride)
        .take(resized.0 as usize)
        .flat_map(|row| row[..resized.1 as usize].to_vec())
        .collect::<Vec<_>>();
    resize_logits(&cropped, resized, target)
}

fn rgb_image(shape: [u32; 4], image: &[u8]) -> RgbImage {
    let [_, _, height, width] = shape;
    let plane = (width * height) as usize;
    RgbImage::from_fn(width, height, |x, y| {
        let index = (y * width + x) as usize;
        Rgb([image[index], image[plane + index], image[2 * plane + index]])
    })
}

/// Bilinear resample of one logits mask; `(height, width)` on both sides. The `image` crate's
/// resizer clamps `f32` to `[0, 1]`, which would destroy unbounded logits, so mask resampling
/// follows OpenCV's `INTER_LINEAR` convention here: half-pixel centres and clamped edges.
fn resize_logits(mask: &[f32], source: (u32, u32), target: (u32, u32)) -> Vec<f32> {
    let (source_height, source_width) = (source.0 as usize, source.1 as usize);
    let ratios = (
        source.0 as f32 / target.0 as f32,
        source.1 as f32 / target.1 as f32,
    );
    let axis = |index: u32, ratio: f32, limit: usize| {
        let position = ((index as f32 + 0.5) * ratio - 0.5).clamp(0.0, limit as f32 - 1.0);
        let low = position as usize;
        (low, (limit - 1).min(low + 1), position - low as f32)
    };
    let mut resized = Vec::with_capacity(target.0 as usize * target.1 as usize);
    for y in 0..target.0 {
        let (top, bottom, fy) = axis(y, ratios.0, source_height);
        for x in 0..target.1 {
            let (left, right, fx) = axis(x, ratios.1, source_width);
            let blend = |row: usize| {
                mask[row * source_width + left] * (1.0 - fx) + mask[row * source_width + right] * fx
            };
            resized.push(blend(top) * (1.0 - fy) + blend(bottom) * fy);
        }
    }
    resized
}

fn tensor<const N: usize>(shape: [usize; N], values: Vec<f32>) -> Result<Tensor<f32>, String> {
    Tensor::from_array((shape, values)).map_err(|error| format!("invalid_input: {error}"))
}

fn embedding<'a>(values: &'a [f32], shape: &[i64]) -> Result<TensorRef<'a, f32>, String> {
    TensorRef::from_array_view((shape.to_vec(), values))
        .map_err(|error| format!("invalid_input: {error}"))
}

fn extract_embeddings(
    encoded: &SessionOutputs<'_>,
    names: &[&str],
    shapes: &[&[i64]],
) -> Result<Vec<Vec<f32>>, String> {
    names
        .iter()
        .zip(shapes)
        .map(|(name, expected)| {
            let (actual, values) = encoded[*name]
                .try_extract_tensor::<f32>()
                .map_err(|error| format!("invalid_output: {error}"))?;
            if actual.as_ref() != *expected || values.iter().any(|value| !value.is_finite()) {
                return Err("invalid_output: selection encoding differs".into());
            }
            Ok(values.to_vec())
        })
        .collect()
}

/// Reads the decoder's `count` candidate masks over the 256² grid plus their IoU scores.
fn extract_masks<'outputs, 'res>(
    decoded: &'outputs SessionOutputs<'res>,
    name: &str,
    score_shape: [i64; 2],
    count: usize,
) -> Result<(&'outputs [f32], &'outputs [f32]), String> {
    let (mask_shape, masks) = decoded[name]
        .try_extract_tensor::<f32>()
        .map_err(|error| format!("invalid_output: {error}"))?;
    let (actual_scores, scores) = decoded["iou_predictions"]
        .try_extract_tensor::<f32>()
        .map_err(|error| format!("invalid_output: {error}"))?;
    if mask_shape.as_ref() != [1, count as i64, MASK_GRID as i64, MASK_GRID as i64]
        || actual_scores.as_ref() != score_shape
        || masks.iter().chain(scores).any(|value| !value.is_finite())
    {
        return Err("invalid_output: selection output differs".into());
    }
    Ok((masks, scores))
}

pub(super) fn source_size(shape: [u32; 4]) -> Result<(u32, u32), String> {
    let [batch, channels, height, width] = shape;
    if batch != 1 || channels != 3 || u64::from(height) * u64::from(width) > 16_777_216 {
        return Err("invalid_input: selection source exceeds its budget".into());
    }
    Ok((height, width))
}

#[cfg(test)]
#[path = "tests/selection.rs"]
mod tests;
