//! Discovery for flat sheets: sticker packs, icon sets and other sources drawn over one uniform
//! backdrop. Such sources carry their own structure, so the layers are read from pixel
//! connectivity instead of a model.

use super::{components, Decomposition, Layers};
use image::RgbaImage;

/// Width of the border ring sampled to estimate the backdrop.
const RING: u32 = 2;
/// Share of ring pixels that must agree before the backdrop counts as uniform.
const MIN_RING_SHARE: f32 = 0.85;
/// Summed absolute RGB distance that still counts as backdrop.
const COLOR_TOLERANCE: i32 = 36;
const ALPHA_PRESENT: u8 = 16;
const MIN_FOREGROUND_RATIO: f32 = 0.01;
const MAX_FOREGROUND_RATIO: f32 = 0.70;

enum Backdrop {
    Transparent,
    Color([u8; 3]),
}

/// Returns a layer stack when the source is drawn over a uniform backdrop, otherwise `None`. The
/// backdrop is a flat fill, so it is not a layer of its own: the stack is the subjects the sheet
/// carries, read in reading order.
pub(super) fn detect(source: &RgbaImage, max_layers: u8) -> Option<Layers> {
    let backdrop = estimate(source)?;
    let mask = source
        .pixels()
        .map(|pixel| match backdrop {
            Backdrop::Transparent => pixel[3] >= ALPHA_PRESENT,
            Backdrop::Color(color) => distance(&pixel.0, &color) > COLOR_TOLERANCE,
        })
        .collect::<Vec<_>>();
    let ratio = mask.iter().filter(|inside| **inside).count() as f32 / mask.len() as f32;
    if !(MIN_FOREGROUND_RATIO..=MAX_FOREGROUND_RATIO).contains(&ratio) {
        return None;
    }
    let mut found = components::merge(components::find(&mask, source.width(), source.height()));
    // Components arrive largest first so the layer budget keeps the dominant subjects, but labels
    // follow reading order because that is how a sheet is read and named.
    found.truncate(usize::from(max_layers));
    let found = components::reading_order(found);
    let count = u8::try_from(found.len()).ok()?;
    if count == 0 {
        return None;
    }
    let mut labels = vec![0_u8; mask.len()];
    for (rank, component) in found.iter().enumerate() {
        let label = u8::try_from(rank + 1).ok()?;
        for pixel in &component.pixels {
            labels[*pixel as usize] = label;
        }
    }
    Some(Layers {
        backdrop: None,
        alpha: None,
        decomposition: Decomposition {
            labels,
            count,
            width: source.width(),
            height: source.height(),
        },
    })
}

fn estimate(source: &RgbaImage) -> Option<Backdrop> {
    let ring = source
        .enumerate_pixels()
        .filter(|(x, y, _)| {
            *x < RING || *y < RING || *x + RING >= source.width() || *y + RING >= source.height()
        })
        .map(|(_, _, pixel)| pixel.0)
        .collect::<Vec<_>>();
    if ring.is_empty() {
        return None;
    }
    let share = |count: usize| count as f32 / ring.len() as f32;
    let transparent = ring.iter().filter(|pixel| pixel[3] < ALPHA_PRESENT).count();
    if share(transparent) >= MIN_RING_SHARE {
        return Some(Backdrop::Transparent);
    }
    let mut sums = [0_u64; 3];
    for pixel in &ring {
        for (sum, channel) in sums.iter_mut().zip(pixel) {
            *sum += u64::from(*channel);
        }
    }
    let color = sums.map(|sum| (sum / ring.len() as u64) as u8);
    let agreeing = ring
        .iter()
        .filter(|pixel| distance(pixel, &color) <= COLOR_TOLERANCE)
        .count();
    (share(agreeing) >= MIN_RING_SHARE).then_some(Backdrop::Color(color))
}

fn distance(pixel: &[u8; 4], color: &[u8; 3]) -> i32 {
    pixel
        .iter()
        .zip(color)
        .map(|(left, right)| (i32::from(*left) - i32::from(*right)).abs())
        .sum()
}

#[cfg(test)]
#[path = "tests/tiled.rs"]
mod tests;
