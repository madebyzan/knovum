use super::{from_saliency, ImageSelectionPoint, MAX_DISCOVERY_SEEDS};

const EDGE: u32 = 128;

/// A saliency field with a flat, low background and filled subject blobs. Real fields are
/// upsampled from a 320px model output, so a subject is always a blob rather than a lone pixel.
fn saliency(blobs: &[[u32; 4]]) -> Vec<u8> {
    let mut field = vec![4_u8; (EDGE * EDGE) as usize];
    for [left, top, width, height] in blobs {
        for y in *top..top + height {
            for x in *left..left + width {
                field[(y * EDGE + x) as usize] = 250;
            }
        }
    }
    field
}

fn positions(seeds: &[ImageSelectionPoint]) -> Vec<(u32, u32)> {
    seeds
        .iter()
        .map(|seed| ((seed.x * EDGE as f32) as u32, (seed.y * EDGE as f32) as u32))
        .collect()
}

fn inside([left, top, width, height]: [u32; 4], (x, y): (u32, u32)) -> bool {
    (left..left + width).contains(&x) && (top..top + height).contains(&y)
}

#[test]
fn every_subject_is_prompted_before_the_canvas_is_explored() {
    let subjects = [[8, 8, 16, 16], [96, 96, 24, 24]];
    let seeds = from_saliency(&saliency(&subjects), (EDGE, EDGE));
    let placed = positions(&seeds);
    assert!(seeds.iter().all(|seed| seed.foreground));
    assert!(
        seeds
            .iter()
            .all(|seed| (0.0..=1.0).contains(&seed.x) && (0.0..=1.0).contains(&seed.y)),
        "prompts are normalised"
    );
    // The largest subject is prompted first, then the smaller one, then exploration.
    assert!(inside(subjects[1], placed[0]), "largest subject first");
    assert!(inside(subjects[0], placed[1]));
    assert!(
        placed[2..]
            .iter()
            .all(|point| !inside(subjects[0], *point) && !inside(subjects[1], *point)),
        "exploration stays out of the subjects it already prompted"
    );
}

#[test]
fn two_subjects_sharing_one_region_are_prompted_separately() {
    // Both blobs sit inside the same exploration cell, which a grid of cell peaks would collapse
    // into a single prompt and therefore a single layer.
    let subjects = [[2, 2, 5, 5], [10, 10, 5, 5]];
    let seeds = from_saliency(&saliency(&subjects), (EDGE, EDGE));
    let placed = positions(&seeds);
    assert!(
        placed.iter().any(|point| inside(subjects[0], *point))
            && placed.iter().any(|point| inside(subjects[1], *point)),
        "each subject carries its own prompt"
    );
}

#[test]
fn one_wide_subject_is_prompted_once() {
    let subject = [16, 60, 96, 12];
    let seeds = from_saliency(&saliency(&[subject]), (EDGE, EDGE));
    let inside_subject = positions(&seeds)
        .into_iter()
        .filter(|point| inside(subject, *point))
        .count();
    assert_eq!(
        inside_subject, 1,
        "a subject spanning several cells is one layer, not one per cell"
    );
}

#[test]
fn a_field_without_subjects_explores_rather_than_giving_up() {
    // Saliency reports a single foreground. A flat field means it found none, which is exactly
    // when the secondary objects of a scene need prompting.
    let seeds = from_saliency(&saliency(&[]), (EDGE, EDGE));
    assert_eq!(seeds.len(), MAX_DISCOVERY_SEEDS);
}

#[test]
fn the_seed_budget_is_never_exceeded() {
    let crowd = (0..64)
        .map(|index| [index % 8 * 16 + 2, index / 8 * 16 + 2, 10, 10])
        .collect::<Vec<_>>();
    let seeds = from_saliency(&saliency(&crowd), (EDGE, EDGE));
    assert!(!seeds.is_empty() && seeds.len() <= MAX_DISCOVERY_SEEDS);
}

#[test]
fn a_field_that_does_not_match_its_source_is_ignored() {
    assert!(from_saliency(&vec![250; 10], (EDGE, EDGE)).is_empty());
    assert!(from_saliency(&saliency(&[]), (0, EDGE)).is_empty());
}
