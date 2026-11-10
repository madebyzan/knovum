use super::session;
use crate::cancellation::Cancellation;
use image::{
    imageops::{self, FilterType},
    GrayImage, Luma, Rgb, RgbImage,
};
use knovum_inference_contracts::{
    image_compute::ImageModelVariant,
    image_spec::{Geometry, LAMA_SPEC},
};
use ort::{session::Session, value::Tensor};
use std::path::Path;

/// Big LaMa's graph reads one fixed square.
const LAMA_EDGE: u32 = match LAMA_SPEC.geometry {
    Geometry::FittedWindow { edge, .. } => edge,
    _ => 512,
};
/// Context kept on every side of a hole, as a share of the hole's longer side: the model continues
/// what surrounds the hole, so it must see enough of it.
const CONTEXT_SHARE: f32 = match LAMA_SPEC.geometry {
    Geometry::FittedWindow { context_share, .. } => context_share,
    _ => 0.5,
};
const MIN_CONTEXT: u32 = match LAMA_SPEC.geometry {
    Geometry::FittedWindow { min_context, .. } => min_context,
    _ => 32,
};

/// How a model takes the source. MI-GAN's pipeline export reads the source at any size; Big LaMa
/// reads a fixed square, so the hole is painted inside a context window fitted to that square.
#[derive(Clone, Copy)]
enum Reading {
    WholeSource,
    FittedWindow,
}

pub(super) struct Inpainting {
    session: Session,
    reading: Reading,
    output: &'static str,
}

impl Inpainting {
    pub(super) fn new(
        directory: &Path,
        variant: ImageModelVariant,
        threads: u8,
    ) -> Result<Self, String> {
        let (file, output, reading) = match variant {
            ImageModelVariant::Migan => ("migan_pipeline_v2.onnx", "result", Reading::WholeSource),
            ImageModelVariant::Lama => ("lama_fp32.onnx", "output", Reading::FittedWindow),
            _ => return Err("runtime_mismatch: model does not inpaint".into()),
        };
        Ok(Self {
            session: session::load(
                &directory.join(file),
                threads,
                &["image", "mask"],
                &[output],
            )?,
            reading,
            output,
        })
    }

    /// Paints the masked pixels (`255`) of a planar RGB source and returns the source with only
    /// those pixels replaced.
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
        let painted = match self.reading {
            Reading::WholeSource => self.whole_source(shape, image, mask, cancellation)?,
            Reading::FittedWindow => {
                self.fitted_window((width, height), image, mask, cancellation)?
            }
        };
        composite(image, &painted, mask)
    }

    fn whole_source(
        &mut self,
        shape: [u32; 4],
        image: &[u8],
        mask: &[u8],
        cancellation: &Cancellation,
    ) -> Result<Vec<u8>, String> {
        let [_, _, height, width] = shape;
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
        let (actual, pixels) = outputs[self.output]
            .try_extract_tensor::<u8>()
            .map_err(|error| format!("invalid_output: {error}"))?;
        if actual.as_ref() != shape.map(i64::from) {
            return Err("invalid_output: inpainting output shape differs".into());
        }
        Ok(pixels.to_vec())
    }

    /// Paints the hole inside a window around it, resampled to the model's square and back. A hole
    /// small enough is painted at the source's own resolution; a larger one is read at the scale
    /// that fits it with its context.
    fn fitted_window(
        &mut self,
        (width, height): (u32, u32),
        image: &[u8],
        mask: &[u8],
        cancellation: &Cancellation,
    ) -> Result<Vec<u8>, String> {
        let hole = hole_bounds(mask, width)?;
        let [left, top, right, bottom] = context_window(hole, (width, height));
        let (window_width, window_height) = (right - left, bottom - top);
        let plane = (width * height) as usize;
        let at = |x: u32, y: u32| ((top + y) * width + left + x) as usize;
        let window = RgbImage::from_fn(window_width, window_height, |x, y| {
            let index = at(x, y);
            Rgb([image[index], image[plane + index], image[2 * plane + index]])
        });
        let holes = GrayImage::from_fn(window_width, window_height, |x, y| Luma([mask[at(x, y)]]));
        let fitted = imageops::resize(&window, LAMA_EDGE, LAMA_EDGE, FilterType::Triangle);
        // Any coverage of a hole after resampling is still hole: a sliver of it left as context
        // would carry the removed content back in.
        let fitted_holes = imageops::resize(&holes, LAMA_EDGE, LAMA_EDGE, FilterType::Triangle);
        let edge = LAMA_EDGE as usize;
        let input = Tensor::from_array((
            [1usize, 3, edge, edge],
            (0..3)
                .flat_map(|channel| {
                    fitted
                        .pixels()
                        .map(move |pixel| f32::from(pixel[channel]) / 255.0)
                })
                .collect::<Vec<_>>(),
        ))
        .map_err(|error| format!("invalid_input: {error}"))?;
        let hole_input = Tensor::from_array((
            [1usize, 1, edge, edge],
            fitted_holes
                .pixels()
                .map(|pixel| if pixel[0] > 0 { 1.0_f32 } else { 0.0 })
                .collect::<Vec<_>>(),
        ))
        .map_err(|error| format!("invalid_input: {error}"))?;
        let options = cancellation.run_options()?;
        let outputs = self
            .session
            .run_with_options(
                ort::inputs!["image" => input, "mask" => hole_input],
                &options,
            )
            .map_err(|error| format!("invalid_output: {error}"))?;
        cancellation.check()?;
        let (actual, values) = outputs[self.output]
            .try_extract_tensor::<f32>()
            .map_err(|error| format!("invalid_output: {error}"))?;
        if actual.as_ref() != [1, 3, i64::from(LAMA_EDGE), i64::from(LAMA_EDGE)]
            || values.iter().any(|value| !value.is_finite())
        {
            return Err("invalid_output: inpainting output shape differs".into());
        }
        let square = edge * edge;
        let painted_square = RgbImage::from_fn(LAMA_EDGE, LAMA_EDGE, |x, y| {
            let index = y as usize * edge + x as usize;
            Rgb([0, 1, 2]
                .map(|channel| values[channel * square + index].round().clamp(0.0, 255.0) as u8))
        });
        let painted_window = imageops::resize(
            &painted_square,
            window_width,
            window_height,
            FilterType::CatmullRom,
        );
        let mut painted = image.to_vec();
        for (x, y, pixel) in painted_window.enumerate_pixels() {
            let index = at(x, y);
            for (channel, value) in pixel.0.iter().enumerate() {
                painted[channel * plane + index] = *value;
            }
        }
        Ok(painted)
    }
}

/// Inclusive bounds of the hole; a mask must be binary and hold at least one hole pixel.
fn hole_bounds(mask: &[u8], width: u32) -> Result<[u32; 4], String> {
    validate_mask(mask)?;
    Ok(mask
        .iter()
        .enumerate()
        .filter(|(_, value)| **value == 255)
        .fold([u32::MAX, u32::MAX, 0, 0], |[l, t, r, b], (index, _)| {
            let (x, y) = (index as u32 % width, index as u32 / width);
            [l.min(x), t.min(y), r.max(x), b.max(y)]
        }))
}

/// Half-open `[left, top, right, bottom)` window the hole is painted in: the hole with context on
/// every side, at least the model's square where the source allows, kept inside the source.
pub(super) fn context_window(
    [left, top, right, bottom]: [u32; 4],
    (width, height): (u32, u32),
) -> [u32; 4] {
    let longer = (right - left + 1).max(bottom - top + 1);
    let context = ((longer as f32 * CONTEXT_SHARE) as u32).max(MIN_CONTEXT);
    let span = |low: u32, high: u32, limit: u32| {
        let length = (high - low + 1 + 2 * context).max(LAMA_EDGE).min(limit);
        let centre = (low + high).div_ceil(2);
        let start = centre.saturating_sub(length / 2).min(limit - length);
        (start, start + length)
    };
    let (x0, x1) = span(left, right, width);
    let (y0, y1) = span(top, bottom, height);
    [x0, y0, x1, y1]
}

fn validate_mask(mask: &[u8]) -> Result<(), String> {
    if !mask.contains(&255) || mask.iter().any(|value| !matches!(value, 0 | 255)) {
        return Err("invalid_input: inpainting requires a nonempty binary mask".into());
    }
    Ok(())
}

/// MI-GAN reads the pixels to keep (`255`) rather than the pixels to paint.
pub(super) fn known_mask(mask: &[u8]) -> Result<Vec<u8>, String> {
    validate_mask(mask)?;
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
