//! Pure refinement of the masks discovery returns. A promptable segmenter answers "what belongs to
//! the thing under this point", which is close to — but not exactly — a layer: it may also paint
//! stray specks of similar texture elsewhere on the canvas, leave pinholes where a highlight
//! fooled it, and stops at a hard binary edge. Each of those is a property of the mask rather than
//! of the subject, so it is corrected here before any pixel is cut.

use super::Decomposition;
use crate::spaces::derivation::regions;

/// A part of a layer smaller than this share of the layer's largest part is a stray the decoder
/// painted elsewhere, not part of the subject. Keeping it would also stretch the layer's crop
/// across the canvas.
const MIN_FRAGMENT_SHARE: f32 = 0.05;
/// A hole smaller than this share of its layer is a pinhole in the mask rather than something the
/// subject shows the backdrop through.
const MAX_PINHOLE_SHARE: f32 = 0.01;

/// Drops every layer's stray fragments, fills its pinholes and removes the layers left empty, so
/// labels stay `1..=count` in stack order. Only unassigned pixels ever fill a pinhole: a hole that
/// another layer shows through is that layer's, not a defect.
pub(super) fn refine(mut decomposition: Decomposition) -> Decomposition {
    let width = decomposition.width;
    for (label, frame) in frames(&decomposition) {
        drop_fragments(&mut decomposition.labels, width, frame, label);
        fill_pinholes(&mut decomposition.labels, width, frame, label);
    }
    compact(decomposition)
}

/// Every layer with its bounds grown by one pixel where the canvas allows, so a hole inside the
/// layer never touches the frame it is searched in. One pass over the canvas finds them all.
fn frames(decomposition: &Decomposition) -> Vec<(u8, [u32; 4])> {
    let (width, height) = (decomposition.width, decomposition.height);
    let mut bounds: Vec<Option<[u32; 4]>> = vec![None; usize::from(decomposition.count) + 1];
    for (index, label) in decomposition.labels.iter().enumerate() {
        if *label == 0 {
            continue;
        }
        let (x, y) = (index as u32 % width, index as u32 / width);
        let entry = &mut bounds[usize::from(*label)];
        *entry = Some(entry.map_or([x, y, x, y], |[l, t, r, b]| {
            [l.min(x), t.min(y), r.max(x), b.max(y)]
        }));
    }
    bounds
        .into_iter()
        .enumerate()
        .filter_map(|(label, found)| {
            let [left, top, right, bottom] = found?;
            Some((
                label as u8,
                [
                    left.saturating_sub(1),
                    top.saturating_sub(1),
                    (right + 1).min(width - 1),
                    (bottom + 1).min(height - 1),
                ],
            ))
        })
        .collect()
}

/// Visits the frame as its own small canvas: `local` is one entry per frame pixel and `global`
/// maps a frame index back to the canvas.
fn window(labels: &[u8], width: u32, [left, top, right, bottom]: [u32; 4]) -> Window {
    let (frame_width, frame_height) = (right - left + 1, bottom - top + 1);
    let global = (0..frame_height)
        .flat_map(|y| (0..frame_width).map(move |x| ((top + y) * width + left + x) as usize))
        .collect::<Vec<_>>();
    let local = global.iter().map(|index| labels[*index]).collect();
    Window {
        local,
        global,
        dimensions: (frame_width, frame_height),
    }
}

struct Window {
    local: Vec<u8>,
    global: Vec<usize>,
    dimensions: (u32, u32),
}

fn drop_fragments(labels: &mut [u8], width: u32, frame: [u32; 4], label: u8) {
    let window = window(labels, width, frame);
    let mask = window
        .local
        .iter()
        .map(|at| *at == label)
        .collect::<Vec<_>>();
    let parts = regions::connected(&mask, window.dimensions);
    let Some(largest) = parts.first().map(|part| part.pixels.len()) else {
        return;
    };
    let minimum = largest as f32 * MIN_FRAGMENT_SHARE;
    for part in parts
        .iter()
        .filter(|part| (part.pixels.len() as f32) < minimum)
    {
        for pixel in &part.pixels {
            labels[window.global[*pixel as usize]] = 0;
        }
    }
}

fn fill_pinholes(labels: &mut [u8], width: u32, frame: [u32; 4], label: u8) {
    let window = window(labels, width, frame);
    let outside = window
        .local
        .iter()
        .map(|at| *at != label)
        .collect::<Vec<_>>();
    let area = outside.iter().filter(|out| !**out).count();
    let open = regions::reachable_from_edge(&outside, window.dimensions);
    let enclosed = outside
        .iter()
        .zip(&open)
        .map(|(out, reached)| *out && !reached)
        .collect::<Vec<_>>();
    let maximum = area as f32 * MAX_PINHOLE_SHARE;
    for hole in regions::connected(&enclosed, window.dimensions) {
        let unassigned = hole
            .pixels
            .iter()
            .all(|pixel| window.local[*pixel as usize] == 0);
        if unassigned && (hole.pixels.len() as f32) <= maximum {
            for pixel in &hole.pixels {
                labels[window.global[*pixel as usize]] = label;
            }
        }
    }
}

/// Renumbers the layers that still carry pixels, keeping their stack order.
fn compact(decomposition: Decomposition) -> Decomposition {
    let mut used = vec![false; usize::from(decomposition.count) + 1];
    for label in &decomposition.labels {
        used[usize::from(*label)] = true;
    }
    let mut order = vec![0_u8; used.len()];
    let mut count = 0_u8;
    for (slot, _) in order
        .iter_mut()
        .zip(&used)
        .skip(1)
        .filter(|(_, used)| **used)
    {
        count += 1;
        *slot = count;
    }
    Decomposition {
        labels: decomposition
            .labels
            .iter()
            .map(|label| order[usize::from(*label)])
            .collect(),
        count,
        ..decomposition
    }
}

/// Width of the edge band, in source pixels, over which the salient layer takes saliency's soft
/// coverage. Saliency is read at 320px, so this is about two of its pixels at the source's scale.
fn edge_band((width, height): (u32, u32)) -> u32 {
    (width.max(height) / 160).max(2)
}

/// The layer that holds the salient subject, with an alpha whose edge comes from saliency and
/// whose inside stays solid. Saliency draws a softer, truer edge — hair, fur, motion — than a
/// binary mask, but read across a whole subject it fades wherever the subject is less striking,
/// which would punch holes into it; so it only speaks inside the edge band.
pub(super) fn salient_edge(
    decomposition: &Decomposition,
    saliency: &[u8],
) -> Option<(u8, Vec<u8>)> {
    if saliency.len() != decomposition.labels.len() {
        return None;
    }
    let owner = (1..=decomposition.count).max_by_key(|label| {
        decomposition
            .labels
            .iter()
            .zip(saliency)
            .filter(|(at, _)| *at == label)
            .map(|(_, coverage)| u64::from(*coverage))
            .sum::<u64>()
    })?;
    let dimensions = (decomposition.width, decomposition.height);
    let mask = decomposition
        .labels
        .iter()
        .map(|label| *label == owner)
        .collect::<Vec<_>>();
    let inside = regions::erode(&mask, dimensions, edge_band(dimensions));
    let alpha = mask
        .iter()
        .zip(&inside)
        .zip(saliency)
        .map(|((owned, solid), soft)| match (owned, solid) {
            (false, _) => 0,
            (true, true) => 255,
            (true, false) => *soft,
        })
        .collect();
    Some((owner, alpha))
}

/// The backdrop pixels a completion pass must paint: every subject, grown by a margin. A mask
/// stops at the subject's edge, but the antialiased rim, a motion smear or a contact shadow still
/// carries the subject's colour just outside it; left as known context, completion would extend
/// that colour into the backdrop as a ghost outline of what was lifted.
pub(super) fn backdrop_holes(decomposition: &Decomposition) -> Vec<bool> {
    let dimensions = (decomposition.width, decomposition.height);
    let subjects = decomposition
        .labels
        .iter()
        .map(|label| *label != 0)
        .collect::<Vec<_>>();
    let margin = (decomposition.width.min(decomposition.height) / 100).max(3);
    regions::dilate(&subjects, dimensions, margin)
}

#[cfg(test)]
#[path = "tests/matte.rs"]
mod tests;
