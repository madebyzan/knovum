//! Lifts a sticker sheet off the plain backdrop it was drawn over. The backdrop is what the sheet
//! edge reaches through backdrop-coloured pixels, not every pixel of the backdrop's colour: a white
//! belly, an eye highlight or the inside of a speech bubble is enclosed by the sticker's outline,
//! so it is part of the sticker however white it is.

use image::{Rgba, RgbaImage};

use crate::spaces::derivation::regions;

/// Width of the border ring sampled to read the backdrop.
const RING: u32 = 2;
/// Share of ring pixels that must agree before the backdrop counts as plain.
const MIN_RING_SHARE: f32 = 0.9;
/// Largest per-channel difference from the backdrop colour that is still backdrop. Compression
/// noise stays well inside it; the palest tint a sticker is drawn in does not.
const TOLERANCE: u8 = 24;
/// Difference from the backdrop at which a rim pixel is fully the sticker's. Between `TOLERANCE`
/// and this, a rim pixel is a blend of the two and keeps only the sticker's share.
const OPAQUE_DIFFERENCE: u8 = 96;
/// Alpha below which a pixel of a transparent sheet is backdrop.
const ALPHA_PRESENT: u8 = 16;

enum Backdrop {
    Transparent,
    Color([u8; 3]),
}

/// The sheet with its backdrop made transparent. The sticker rim against a coloured backdrop is
/// un-blended — its alpha is the sticker's coverage and its colour the sticker's own — so a sticker
/// set on any chat background shows no fringe of the sheet it was cut from.
pub(super) fn lift(source: &RgbaImage) -> Result<RgbaImage, String> {
    let backdrop = estimate(source)
        .ok_or("This image is not a sticker sheet: its backdrop is not one plain colour")?;
    let dimensions = (source.width(), source.height());
    let Backdrop::Color(color) = backdrop else {
        let mut lifted = source.clone();
        for pixel in lifted.pixels_mut() {
            if pixel[3] < ALPHA_PRESENT {
                *pixel = Rgba([0, 0, 0, 0]);
            }
        }
        return Ok(lifted);
    };
    let plain = source
        .pixels()
        .map(|pixel| difference(pixel, color) <= TOLERANCE)
        .collect::<Vec<_>>();
    let outside = regions::reachable_from_edge(&plain, dimensions);
    let rim = regions::dilate(&outside, dimensions, 1);
    let mut lifted = source.clone();
    for ((pixel, outside), rim) in lifted.pixels_mut().zip(&outside).zip(&rim) {
        *pixel = match (outside, rim) {
            (true, _) => Rgba([0, 0, 0, 0]),
            (false, true) => unblend(*pixel, color),
            (false, false) => *pixel,
        };
    }
    Ok(lifted)
}

/// Separates a rim pixel into the sticker's colour and coverage, reading the blend as
/// `pixel = coverage × sticker + (1 − coverage) × backdrop`.
fn unblend(pixel: Rgba<u8>, backdrop: [u8; 3]) -> Rgba<u8> {
    let span = f32::from(OPAQUE_DIFFERENCE - TOLERANCE);
    let coverage =
        ((f32::from(difference(&pixel, backdrop)) - f32::from(TOLERANCE)) / span).clamp(0.0, 1.0);
    if coverage == 0.0 {
        return Rgba([0, 0, 0, 0]);
    }
    let channel = |index: usize| {
        let (seen, behind) = (f32::from(pixel[index]), f32::from(backdrop[index]));
        (behind + (seen - behind) / coverage)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Rgba([
        channel(0),
        channel(1),
        channel(2),
        (f32::from(pixel[3]) * coverage).round() as u8,
    ])
}

/// Reads the backdrop off the border ring: transparent, one colour, or — when the ring disagrees
/// with itself — not a plain backdrop at all.
fn estimate(source: &RgbaImage) -> Option<Backdrop> {
    let (width, height) = source.dimensions();
    let ring = source
        .enumerate_pixels()
        .filter(|(x, y, _)| *x < RING || *y < RING || x + RING >= width || y + RING >= height)
        .map(|(_, _, pixel)| *pixel)
        .collect::<Vec<_>>();
    if ring.is_empty() {
        return None;
    }
    let share = |count: usize| count as f32 / ring.len() as f32;
    if share(ring.iter().filter(|pixel| pixel[3] < ALPHA_PRESENT).count()) >= MIN_RING_SHARE {
        return Some(Backdrop::Transparent);
    }
    let color = [0, 1, 2].map(|channel| {
        let mut values = ring.iter().map(|pixel| pixel[channel]).collect::<Vec<_>>();
        values.sort_unstable();
        values[values.len() / 2]
    });
    let agreeing = ring
        .iter()
        .filter(|pixel| pixel[3] >= ALPHA_PRESENT && difference(pixel, color) <= TOLERANCE)
        .count();
    (share(agreeing) >= MIN_RING_SHARE).then_some(Backdrop::Color(color))
}

fn difference(pixel: &Rgba<u8>, color: [u8; 3]) -> u8 {
    (0..3)
        .map(|channel| pixel[channel].abs_diff(color[channel]))
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "tests/backdrop.rs"]
mod tests;
