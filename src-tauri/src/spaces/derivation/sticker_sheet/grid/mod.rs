//! Reads the grid a sticker sheet is laid out on. Rows are read across the whole sheet, then each
//! row is read into its own columns, so a row holding fewer stickers, or set off centre, is read
//! as it is.
//!
//! Along one axis a sheet is a lattice of evenly pitched cells (see [`lattice`]). The blank lines
//! of its foreground profile are where a cut may lie: the gutters between stickers, but also the
//! gap between a character and its caption or prop. The lattice tells the two apart.

mod lattice;

use std::iter;

use lattice::Lattice;

/// Most rows a sheet is read as, and most stickers a row is read as.
pub(super) const MAX_SPLITS: u32 = 8;
/// A line whose foreground stays at or below this share of its length — and at least one pixel —
/// is blank: compression ringing or a stray dot does not close a gutter.
const BLANK_SHARE: f32 = 0.002;

/// Half-open `[start, end)` span of lines along one axis.
type Span = (u32, u32);

/// The content bounds, inclusive `[left, top, right, bottom]`, of every cell that holds a sticker,
/// in reading order.
pub(super) fn cells(mask: &[bool], (width, height): (u32, u32)) -> Vec<[u32; 4]> {
    let rows = (0..height)
        .map(|y| line(mask, width, (0..width).map(|x| (x, y))))
        .collect::<Vec<_>>();
    split(&rows, width)
        .into_iter()
        .flat_map(|(top, bottom)| {
            let columns = (0..width)
                .map(|x| line(mask, width, (top..bottom).map(move |y| (x, y))))
                .collect::<Vec<_>>();
            split(&columns, bottom - top)
                .into_iter()
                .filter_map(move |(left, right)| bounds(mask, width, [left, top, right, bottom]))
        })
        .collect()
}

fn line(mask: &[bool], width: u32, pixels: impl Iterator<Item = (u32, u32)>) -> u32 {
    pixels
        .filter(|(x, y)| mask[(y * width + x) as usize])
        .count() as u32
}

/// The drawn spans one profile is read as: the most cells a lattice gives, or the whole content
/// as one when none of two or more fits. With nothing drawn there are none.
fn split(profile: &[u32], across: u32) -> Vec<Span> {
    let limit = (across as f32 * BLANK_SHARE).ceil().max(1.0) as u32;
    let runs = blank_runs(profile, limit);
    if runs.is_empty() {
        return Vec::new();
    }
    (2..=MAX_SPLITS)
        .rev()
        .find_map(|count| Lattice::new(&runs, count).read())
        .unwrap_or_else(|| vec![(runs[0].1, runs[runs.len() - 1].0)])
}

/// Maximal runs of blank lines in order: the leading margin, every gap between drawn lines, the
/// trailing margin. Either margin may be empty, at a sheet edge. With nothing drawn there are
/// none.
fn blank_runs(profile: &[u32], limit: u32) -> Vec<Span> {
    let drawn = (0..)
        .zip(profile)
        .filter(|(_, count)| **count > limit)
        .map(|(at, _)| at)
        .collect::<Vec<u32>>();
    let (Some(first), Some(last)) = (drawn.first(), drawn.last()) else {
        return Vec::new();
    };
    let gaps = drawn
        .windows(2)
        .filter(|pair| pair[1] > pair[0] + 1)
        .map(|pair| (pair[0] + 1, pair[1]));
    iter::once((0, *first))
        .chain(gaps)
        .chain(iter::once((last + 1, profile.len() as u32)))
        .collect()
}

/// Inclusive bounds of the foreground inside the half-open cell `[left, top, right, bottom)`.
fn bounds(mask: &[bool], width: u32, [left, top, right, bottom]: [u32; 4]) -> Option<[u32; 4]> {
    (top..bottom)
        .flat_map(|y| (left..right).map(move |x| (x, y)))
        .filter(|(x, y)| mask[(y * width + x) as usize])
        .fold(None, |found, (x, y)| {
            Some(found.map_or([x, y, x, y], |[l, t, r, b]: [u32; 4]| {
                [l.min(x), t.min(y), r.max(x), b.max(y)]
            }))
        })
}

#[cfg(test)]
mod tests;
