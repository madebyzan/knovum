use super::session;
use crate::cancellation::Cancellation;
use image::{
    imageops::{self, FilterType},
    GrayImage, Luma, Rgb, RgbImage,
};
use knovum_inference_contracts::image_spec::LAMA_WINDOW;
use ort::{session::Session, value::Tensor};
use std::{collections::VecDeque, path::Path};

pub(super) struct Inpainting {
    session: Session,
}

impl Inpainting {
    pub(super) fn new(directory: &Path, threads: u8) -> Result<Self, String> {
        Ok(Self {
            session: session::load(
                &directory.join("lama_fp32.onnx"),
                threads,
                &["image", "mask"],
                &["output"],
            )?,
        })
    }

    /// Paints the masked pixels (`255`) of a planar RGB source. Every group of nearby holes is
    /// painted in its own window, so a second hole elsewhere never forces both to be read at a
    /// coarser scale.
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
            || mask.len() != width as usize * height as usize
        {
            return Err("invalid_input: inpainting tensor shapes differ".into());
        }
        validate_mask(mask)?;
        let mut painted = image.to_vec();
        for window in windows(mask, (width, height)) {
            cancellation.check()?;
            self.paint(&mut painted, (width, height), mask, window, cancellation)?;
        }
        Ok(painted)
    }

    /// Paints the holes inside `window`, read at the scale that fits the window into the model's
    /// square, and blends the result back over the holes and a feather just outside them.
    fn paint(
        &mut self,
        image: &mut [u8],
        (width, height): (u32, u32),
        mask: &[u8],
        [left, top, right, bottom]: [u32; 4],
        cancellation: &Cancellation,
    ) -> Result<(), String> {
        let (window_width, window_height) = (right - left, bottom - top);
        let plane = (width * height) as usize;
        let at = |x: u32, y: u32| ((top + y) * width + left + x) as usize;
        let window = RgbImage::from_fn(window_width, window_height, |x, y| {
            let index = at(x, y);
            Rgb([image[index], image[plane + index], image[2 * plane + index]])
        });
        let holes = GrayImage::from_fn(window_width, window_height, |x, y| Luma([mask[at(x, y)]]));
        let edge = LAMA_WINDOW.edge;
        let fitted = imageops::resize(&window, edge, edge, FilterType::Triangle);
        // Any coverage of a hole after resampling is still hole: a sliver of it left as context
        // would carry the removed content back in.
        let fitted_holes = imageops::resize(&holes, edge, edge, FilterType::Triangle);
        let side = edge as usize;
        let input = Tensor::from_array((
            [1usize, 3, side, side],
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
            [1usize, 1, side, side],
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
        let (actual, values) = outputs["output"]
            .try_extract_tensor::<f32>()
            .map_err(|error| format!("invalid_output: {error}"))?;
        if actual.as_ref() != [1, 3, i64::from(edge), i64::from(edge)]
            || values.iter().any(|value| !value.is_finite())
        {
            return Err("invalid_output: inpainting output shape differs".into());
        }
        // Big LaMa returns RGB in `[0, 255]`.
        let square = side * side;
        let painted_square = RgbImage::from_fn(edge, edge, |x, y| {
            let index = y as usize * side + x as usize;
            Rgb([0, 1, 2]
                .map(|channel| values[channel * square + index].round().clamp(0.0, 255.0) as u8))
        });
        let painted = imageops::resize(
            &painted_square,
            window_width,
            window_height,
            FilterType::CatmullRom,
        );
        let weights = feather(
            holes.as_raw(),
            (window_width, window_height),
            feather_width(window_width.max(window_height)),
        );
        for (index, (x, y, pixel)) in painted.enumerate_pixels().enumerate() {
            let weight = weights[index];
            if weight == 0.0 {
                continue;
            }
            let target = at(x, y);
            for (channel, value) in pixel.0.iter().enumerate() {
                let original = &mut image[channel * plane + target];
                *original = (f32::from(*original) * (1.0 - weight) + f32::from(*value) * weight)
                    .round() as u8;
            }
        }
        Ok(())
    }
}

/// The painted fill was resampled from the model's square, so it is slightly softer than the
/// source around it. Blending over this many pixels outside the hole — one per source pixel a
/// model pixel covers, never less than one — hides the seam without reaching far into what the
/// person kept.
pub(super) fn feather_width(window_edge: u32) -> u32 {
    window_edge.div_ceil(LAMA_WINDOW.edge).max(1)
}

/// How much of the painted fill each window pixel takes: all of it inside a hole, fading linearly
/// over `reach` pixels outside it by chessboard distance, none beyond.
pub(super) fn feather(holes: &[u8], (width, height): (u32, u32), reach: u32) -> Vec<f32> {
    let (width, height) = (width as usize, height as usize);
    let mut distance = holes
        .iter()
        .map(|value| if *value == 255 { 0 } else { u32::MAX })
        .collect::<Vec<_>>();
    let mut frontier = (0..distance.len())
        .filter(|index| distance[*index] == 0)
        .collect::<VecDeque<_>>();
    while let Some(index) = frontier.pop_front() {
        let next = distance[index] + 1;
        if next > reach {
            continue;
        }
        let (x, y) = (index % width, index / width);
        for neighbour_y in y.saturating_sub(1)..=(y + 1).min(height - 1) {
            for neighbour_x in x.saturating_sub(1)..=(x + 1).min(width - 1) {
                let neighbour = neighbour_y * width + neighbour_x;
                if distance[neighbour] > next {
                    distance[neighbour] = next;
                    frontier.push_back(neighbour);
                }
            }
        }
    }
    distance
        .into_iter()
        .map(|steps| match steps {
            0 => 1.0,
            steps if steps <= reach => 1.0 - steps as f32 / (reach + 1) as f32,
            _ => 0.0,
        })
        .collect()
}

/// Half-open windows to paint, one per group of holes whose context would overlap. Holes far
/// apart are painted at their own scale; holes close together share one window, so no window
/// reads another's unpainted hole as context.
pub(super) fn windows(mask: &[u8], (width, height): (u32, u32)) -> Vec<[u32; 4]> {
    let mut groups = hole_regions(mask, width);
    while merge_once(&mut groups, (width, height)) {}
    groups
        .into_iter()
        .map(|bounds| context_window(bounds, (width, height)))
        .collect()
}

/// Merges the first pair of groups whose windows intersect; `false` once none do.
fn merge_once(groups: &mut Vec<[u32; 4]>, dimensions: (u32, u32)) -> bool {
    for first in 0..groups.len() {
        for second in first + 1..groups.len() {
            let a = context_window(groups[first], dimensions);
            let b = context_window(groups[second], dimensions);
            if a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3] {
                let other = groups.swap_remove(second);
                let group = &mut groups[first];
                *group = [
                    group[0].min(other[0]),
                    group[1].min(other[1]),
                    group[2].max(other[2]),
                    group[3].max(other[3]),
                ];
                return true;
            }
        }
    }
    false
}

/// Inclusive bounds of every 8-connected hole region.
fn hole_regions(mask: &[u8], width: u32) -> Vec<[u32; 4]> {
    let width = width as usize;
    let height = mask.len() / width;
    let mut seen = vec![false; mask.len()];
    let mut regions = Vec::new();
    for start in 0..mask.len() {
        if mask[start] != 255 || seen[start] {
            continue;
        }
        seen[start] = true;
        let mut queue = VecDeque::from([start]);
        let (x, y) = ((start % width) as u32, (start / width) as u32);
        let mut bounds = [x, y, x, y];
        while let Some(index) = queue.pop_front() {
            let (x, y) = (index % width, index / width);
            bounds = [
                bounds[0].min(x as u32),
                bounds[1].min(y as u32),
                bounds[2].max(x as u32),
                bounds[3].max(y as u32),
            ];
            for neighbour_y in y.saturating_sub(1)..=(y + 1).min(height - 1) {
                for neighbour_x in x.saturating_sub(1)..=(x + 1).min(width - 1) {
                    let neighbour = neighbour_y * width + neighbour_x;
                    if mask[neighbour] == 255 && !seen[neighbour] {
                        seen[neighbour] = true;
                        queue.push_back(neighbour);
                    }
                }
            }
        }
        regions.push(bounds);
    }
    regions
}

/// Half-open `[left, top, right, bottom)` window a hole is painted in: the hole with context on
/// every side, at least the model's square where the source allows, kept inside the source.
pub(super) fn context_window(
    [left, top, right, bottom]: [u32; 4],
    (width, height): (u32, u32),
) -> [u32; 4] {
    let longer = (right - left + 1).max(bottom - top + 1);
    let context = ((longer as f32 * LAMA_WINDOW.context_share) as u32).max(LAMA_WINDOW.min_context);
    let span = |low: u32, high: u32, limit: u32| {
        let length = (high - low + 1 + 2 * context)
            .max(LAMA_WINDOW.edge)
            .min(limit);
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
