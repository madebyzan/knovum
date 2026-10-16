use super::selection::{self, Selection};
use crate::cancellation::Cancellation;
use knovum_inference_contracts::image_compute::ImageSelectionPoint;

/// Seeds are probed at this long edge. Discovery only needs mask overlap statistics, so small
/// masks keep the sweep cheap; survivors are decoded again at source resolution.
const PROBE_EDGE: u32 = 192;
/// Logit offset used for the stability score. A mask that barely survives re-thresholding sits on
/// the decoder's decision boundary and is not a usable layer.
const STABILITY_OFFSET: f32 = 1.0;
const MIN_SCORE: f32 = 0.6;
const MIN_STABILITY: f32 = 0.85;
const MIN_AREA_RATIO: f32 = 0.002;
const MAX_AREA_RATIO: f32 = 0.95;
/// Overlap above which the weaker candidate describes the same object as a stronger one.
const MAX_OVERLAP: f32 = 0.5;
/// Containment above which the weaker candidate is a part of a stronger one rather than a peer.
const MAX_CONTAINMENT: f32 = 0.85;
pub(super) const MAX_SEEDS: usize = 64;
pub(super) const MAX_LAYERS: u8 = 24;

struct Candidate {
    seed: usize,
    mask: Vec<bool>,
    area: usize,
    rank: f32,
}

/// Returns a label map over the source: `0` is unassigned and `1..=n` are candidate ranks, best
/// first. Labels are disjoint, so the host can cut one layer per label without double painting.
pub(super) fn discover(
    selection: &mut Selection,
    shape: [u32; 4],
    image: &[u8],
    identity: String,
    (seeds, max_layers): (&[ImageSelectionPoint], u8),
    cancellation: &Cancellation,
) -> Result<Vec<u8>, String> {
    let (height, width) = selection::source_size(shape)?;
    if seeds.is_empty() || seeds.len() > MAX_SEEDS || !(1..=MAX_LAYERS).contains(&max_layers) {
        return Err("invalid_input: layer discovery seed or layer budget is invalid".into());
    }
    selection.encode(shape, image, identity, cancellation)?;
    let probe = probe_size(height, width);
    let mut candidates = Vec::new();
    for (seed, point) in seeds.iter().enumerate() {
        cancellation.check()?;
        if let Some(candidate) = probe_seed(selection, point, seed, probe, cancellation)? {
            candidates.push(candidate);
        }
    }
    candidates.sort_by(|left, right| right.rank.total_cmp(&left.rank));
    let kept = elect(candidates, max_layers);
    paint(selection, seeds, &kept, (height, width), cancellation)
}

fn probe_seed(
    selection: &mut Selection,
    point: &ImageSelectionPoint,
    seed: usize,
    (height, width): (u32, u32),
    cancellation: &Cancellation,
) -> Result<Option<Candidate>, String> {
    let (coords, labels) = super::prompts::foreground_point(point, height, width)?;
    let decoded = selection.decode(coords, labels, (height, width), cancellation)?;
    let mask = decoded.mask(0.0).collect::<Vec<_>>();
    let area = mask.iter().filter(|inside| **inside).count();
    let ratio = area as f32 / mask.len() as f32;
    if decoded.score < MIN_SCORE || !(MIN_AREA_RATIO..=MAX_AREA_RATIO).contains(&ratio) {
        return Ok(None);
    }
    let stability = stability(&decoded);
    if stability < MIN_STABILITY {
        return Ok(None);
    }
    Ok(Some(Candidate {
        seed,
        mask,
        area,
        rank: decoded.score * stability,
    }))
}

/// Intersection over union between the mask thresholded above and below the decision boundary.
fn stability(decoded: &selection::DecodedPrompt) -> f32 {
    let (intersection, union) = decoded
        .mask(STABILITY_OFFSET)
        .zip(decoded.mask(-STABILITY_OFFSET))
        .fold(
            (0_usize, 0_usize),
            |(intersection, union), (tight, wide)| {
                (
                    intersection + usize::from(tight && wide),
                    union + usize::from(tight || wide),
                )
            },
        );
    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

/// Keeps the strongest candidates that describe distinct objects. Candidates arrive best first, so
/// a later candidate only survives when it neither overlaps nor sits inside an elected one.
fn elect(candidates: Vec<Candidate>, max_layers: u8) -> Vec<Candidate> {
    let mut kept: Vec<Candidate> = Vec::new();
    for candidate in candidates {
        if kept.len() == usize::from(max_layers) {
            break;
        }
        if kept
            .iter()
            .all(|elected| distinct(&candidate, elected) && distinct(elected, &candidate))
        {
            kept.push(candidate);
        }
    }
    kept
}

fn distinct(candidate: &Candidate, elected: &Candidate) -> bool {
    let intersection = candidate
        .mask
        .iter()
        .zip(&elected.mask)
        .filter(|(left, right)| **left && **right)
        .count();
    let union = candidate.area + elected.area - intersection;
    let overlap = if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    };
    let containment = if candidate.area == 0 {
        1.0
    } else {
        intersection as f32 / candidate.area as f32
    };
    overlap <= MAX_OVERLAP && containment <= MAX_CONTAINMENT
}

/// Decodes the elected seeds at source resolution and paints their ranks into one label map. The
/// probe masks are too coarse to cut layers from, and the highest ranked label owns shared pixels.
fn paint(
    selection: &mut Selection,
    seeds: &[ImageSelectionPoint],
    kept: &[Candidate],
    (height, width): (u32, u32),
    cancellation: &Cancellation,
) -> Result<Vec<u8>, String> {
    let mut labels = vec![0_u8; height as usize * width as usize];
    for (rank, candidate) in kept.iter().enumerate() {
        cancellation.check()?;
        let point = seeds
            .get(candidate.seed)
            .ok_or("invalid_output: layer discovery seed index differs")?;
        let (coords, prompt_labels) = super::prompts::foreground_point(point, height, width)?;
        let decoded = selection.decode(coords, prompt_labels, (height, width), cancellation)?;
        let label = u8::try_from(rank + 1).map_err(|_| "invalid_output: layer rank overflows")?;
        for (slot, inside) in labels.iter_mut().zip(decoded.mask(0.0)) {
            if inside && *slot == 0 {
                *slot = label;
            }
        }
    }
    Ok(labels)
}

fn probe_size(height: u32, width: u32) -> (u32, u32) {
    let edge = height.max(width);
    if edge <= PROBE_EDGE {
        return (height, width);
    }
    let scale = f64::from(PROBE_EDGE) / f64::from(edge);
    (
        ((f64::from(height) * scale).round() as u32).max(1),
        ((f64::from(width) * scale).round() as u32).max(1),
    )
}

#[cfg(test)]
#[path = "tests/discovery.rs"]
mod tests;
