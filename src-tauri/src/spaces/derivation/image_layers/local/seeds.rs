//! Seed points for layer discovery. EfficientSAM is prompt driven, so the quality of a split
//! depends on where it is asked to look. Saliency is already available at this point, so seeds
//! are placed on salient peaks instead of a blind grid: fewer prompts, and the prompts land on
//! subject centres rather than on subject parts.

use crate::ai::contracts::image_compute::ImageSelectionPoint;

/// Cells per axis. The grid bounds how many prompts one pass can place and keeps them spread.
const GRID: u32 = 6;
/// Saliency a cell peak must reach before it is worth a prompt.
const MIN_PEAK: u8 = 38;
/// Seeds kept beyond the layer budget, so election has alternatives to choose between.
const BUDGET_HEADROOM: usize = 8;

/// Returns normalised foreground prompts, strongest peak first.
pub(super) fn from_saliency(
    saliency: &[u8],
    (width, height): (u32, u32),
    max_layers: u8,
) -> Vec<ImageSelectionPoint> {
    if saliency.len() != (width as usize * height as usize) || width == 0 || height == 0 {
        return Vec::new();
    }
    let mut peaks = (0..GRID * GRID)
        .filter_map(|cell| peak(saliency, (width, height), cell))
        .collect::<Vec<_>>();
    peaks.sort_by(|left, right| right.0.cmp(&left.0));
    peaks.truncate(usize::from(max_layers).saturating_add(BUDGET_HEADROOM));
    peaks
        .into_iter()
        .map(|(_, x, y)| ImageSelectionPoint {
            x: (x as f32 + 0.5) / width as f32,
            y: (y as f32 + 0.5) / height as f32,
            foreground: true,
        })
        .collect()
}

/// The strongest saliency pixel inside one grid cell, when it clears the peak threshold.
fn peak(saliency: &[u8], (width, height): (u32, u32), cell: u32) -> Option<(u8, u32, u32)> {
    let (column, row) = (cell % GRID, cell / GRID);
    let left = column * width / GRID;
    let top = row * height / GRID;
    let right = ((column + 1) * width / GRID).max(left + 1).min(width);
    let bottom = ((row + 1) * height / GRID).max(top + 1).min(height);
    let mut best = (0_u8, left, top);
    for y in top..bottom {
        for x in left..right {
            let value = saliency[(y * width + x) as usize];
            if value > best.0 {
                best = (value, x, y);
            }
        }
    }
    (best.0 >= MIN_PEAK).then_some(best)
}

#[cfg(test)]
#[path = "tests/seeds.rs"]
mod tests;
