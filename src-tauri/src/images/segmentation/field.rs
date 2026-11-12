//! What a background-removal graph reads, and how its answer reads as coverage of the source. Both
//! directions follow the model's specification and nothing else, so each is pinned by tests
//! without a model.

use image::{
    imageops::{self, FilterType},
    RgbaImage,
};

use crate::local_resources::models::catalog::{CoverageField, SegmentationSpec, SquareFit};

/// Where the source sits inside the graph's square, as `[left, top, width, height]`.
pub(super) fn placement(spec: &SegmentationSpec, (width, height): (u32, u32)) -> [u32; 4] {
    let edge = spec.edge;
    match spec.fit {
        SquareFit::Stretch => [0, 0, edge, edge],
        SquareFit::Letterbox => {
            // The longer side fills the square; the shorter keeps the aspect ratio, truncated like
            // the reference implementation.
            let scaled = |short: u32, long: u32| {
                ((u64::from(edge) * u64::from(short) / u64::from(long)) as u32).max(1)
            };
            let (placed_width, placed_height) = if height > width {
                (scaled(width, height), edge)
            } else {
                (edge, scaled(height, width))
            };
            [
                (edge - placed_width) / 2,
                (edge - placed_height) / 2,
                placed_width,
                placed_height,
            ]
        }
    }
}

/// The graph's input: the source's RGB placed in the square and standardized, channel-first.
/// Letterbox padding is black, as the reference pads it.
pub(super) fn model_input(spec: &SegmentationSpec, source: &RgbaImage) -> Vec<f32> {
    let [left, top, width, height] = placement(spec, source.dimensions());
    let placed = imageops::resize(source, width, height, FilterType::Triangle);
    let edge = spec.edge as usize;
    let plane = edge * edge;
    let mut input = vec![0.0; plane * 3];
    for (channel, values) in input.chunks_exact_mut(plane).enumerate() {
        values.fill(spec.standardization.apply(channel, 0));
    }
    for (x, y, pixel) in placed.enumerate_pixels() {
        let index = (top + y) as usize * edge + (left + x) as usize;
        for channel in 0..3 {
            input[channel * plane + index] = spec.standardization.apply(channel, pixel[channel]);
        }
    }
    input
}

/// How much of every source pixel is subject, in `[0, 1]`. The graph's field is read as coverage
/// at its own resolution, the source's placement is cropped from it, and that is resampled
/// bilinearly to the source in floating point, so the model's own soft edge is neither quantized
/// nor sharpened into ringing on the way.
pub(super) fn coverage(
    spec: &SegmentationSpec,
    field: &[f32],
    (width, height): (u32, u32),
) -> Result<Vec<f32>, String> {
    let edge = spec.edge as usize;
    if field.len() != edge * edge {
        return Err("invalid_output: segmentation field has invalid dimensions".into());
    }
    let grid = field
        .iter()
        .map(|&value| match spec.output {
            CoverageField::Logits => 1.0 / (1.0 + (-value).exp()),
            CoverageField::Probability => value.clamp(0.0, 1.0),
        })
        .collect::<Vec<f32>>();
    let [left, top, placed_width, placed_height] = placement(spec, (width, height));
    let (left, top) = (left as usize, top as usize);
    let (placed_width, placed_height) = (placed_width as usize, placed_height as usize);
    let at = |x: usize, y: usize| grid[(top + y) * edge + left + x];
    let axis = |index: u32, target: u32, placed: usize| {
        let position = ((index as f32 + 0.5) * placed as f32 / target as f32 - 0.5)
            .clamp(0.0, (placed - 1) as f32);
        let low = position as usize;
        (low, (low + 1).min(placed - 1), position - low as f32)
    };
    let columns = (0..width)
        .map(|x| axis(x, width, placed_width))
        .collect::<Vec<_>>();
    let mut coverage = Vec::with_capacity(width as usize * height as usize);
    for y in 0..height {
        let (upper, lower, vertical) = axis(y, height, placed_height);
        for &(left_column, right_column, horizontal) in &columns {
            let row = |y: usize| {
                at(left_column, y) * (1.0 - horizontal) + at(right_column, y) * horizontal
            };
            coverage.push(row(upper) * (1.0 - vertical) + row(lower) * vertical);
        }
    }
    Ok(coverage)
}
