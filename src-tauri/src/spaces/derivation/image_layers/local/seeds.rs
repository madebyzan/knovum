//! Seed points for layer discovery. EfficientSAM is prompt driven, so a split can only contain
//! objects that were prompted: an object nobody asks about is painted into the backdrop. The
//! encoder embedding is computed once per source and reused, so one extra seed costs one cheap
//! decoder pass over a small probe. Seeds are therefore spent generously and in two kinds:
//!
//! - **Subject seeds** come from the connected components of the saliency field. One prompt per
//!   component keeps two neighbouring subjects apart and keeps one wide subject together, which a
//!   fixed grid cannot do.
//! - **Exploratory seeds** cover the cells saliency left empty. Saliency reports a single
//!   foreground, so secondary objects — the second car, the cup behind the subject — score below
//!   the subject threshold and would otherwise never be prompted. The worker's own stability and
//!   area filters reject the prompts that land on flat sky or wall, so exploring costs decoder
//!   passes rather than quality.

use super::components;
use crate::ai::contracts::image_compute::{ImageSelectionPoint, MAX_DISCOVERY_SEEDS};

/// Saliency a pixel must reach to belong to a subject component. A cell whose peak stays below it
/// is what exploration is for.
const MIN_SUBJECT: u8 = 38;
/// Cells per axis explored where saliency found no subject.
const EXPLORE_GRID: u32 = 8;

/// Returns normalised foreground prompts: subject seeds first, largest subject first, then the
/// exploratory seeds that cover the rest of the canvas. The layer budget is not a seed budget —
/// the worker elects the layers it publishes out of everything that was prompted.
pub(super) fn from_saliency(
    saliency: &[u8],
    (width, height): (u32, u32),
) -> Vec<ImageSelectionPoint> {
    if saliency.len() != (width as usize * height as usize) || width == 0 || height == 0 {
        return Vec::new();
    }
    let mut seeds = subject_seeds(saliency, (width, height));
    seeds.truncate(MAX_DISCOVERY_SEEDS);
    let room = MAX_DISCOVERY_SEEDS - seeds.len();
    seeds.extend(
        exploratory_seeds(saliency, (width, height))
            .into_iter()
            .take(room),
    );
    seeds
        .into_iter()
        .map(|(x, y)| ImageSelectionPoint {
            x: (x as f32 + 0.5) / width as f32,
            y: (y as f32 + 0.5) / height as f32,
            foreground: true,
        })
        .collect()
}

/// One prompt at the most confident pixel of every salient component, largest component first.
fn subject_seeds(saliency: &[u8], (width, height): (u32, u32)) -> Vec<(u32, u32)> {
    let mask = saliency
        .iter()
        .map(|value| *value >= MIN_SUBJECT)
        .collect::<Vec<_>>();
    components::merge(components::find(&mask, width, height))
        .iter()
        .filter_map(|component| peak(saliency, &component.pixels))
        .map(|index| (index % width, index / width))
        .collect()
}

/// Cell centres of the cells saliency left empty, closest to being a subject first.
fn exploratory_seeds(saliency: &[u8], (width, height): (u32, u32)) -> Vec<(u32, u32)> {
    let mut cells = (0..EXPLORE_GRID * EXPLORE_GRID)
        .filter_map(|cell| {
            let (left, top, right, bottom) = bounds(cell, (width, height));
            let highest = (top..bottom)
                .flat_map(|y| (left..right).map(move |x| (x, y)))
                .map(|(x, y)| saliency[(y * width + x) as usize])
                .max()?;
            (highest < MIN_SUBJECT).then_some((highest, (left + right) / 2, (top + bottom) / 2))
        })
        .collect::<Vec<_>>();
    cells.sort_by_key(|cell| std::cmp::Reverse(cell.0));
    cells.into_iter().map(|(_, x, y)| (x, y)).collect()
}

/// The strongest saliency pixel of one component.
fn peak(saliency: &[u8], pixels: &[u32]) -> Option<u32> {
    pixels
        .iter()
        .copied()
        .max_by_key(|index| saliency[*index as usize])
}

/// Half-open `[left, top, right, bottom)` bounds of one exploratory cell.
fn bounds(cell: u32, (width, height): (u32, u32)) -> (u32, u32, u32, u32) {
    let (column, row) = (cell % EXPLORE_GRID, cell / EXPLORE_GRID);
    let left = column * width / EXPLORE_GRID;
    let top = row * height / EXPLORE_GRID;
    (
        left,
        top,
        ((column + 1) * width / EXPLORE_GRID)
            .max(left + 1)
            .min(width),
        ((row + 1) * height / EXPLORE_GRID).max(top + 1).min(height),
    )
}

#[cfg(test)]
#[path = "tests/seeds.rs"]
mod tests;
