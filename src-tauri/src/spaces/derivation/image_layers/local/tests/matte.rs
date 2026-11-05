use super::{backdrop_holes, refine, salient_edge};
use crate::spaces::derivation::image_layers::local::Decomposition;

const EDGE: u32 = 100;

/// A canvas of labels painted as rectangles `(label, left, top, width, height)`, later ones on top.
fn decomposition(count: u8, blocks: &[(u8, u32, u32, u32, u32)]) -> Decomposition {
    let mut labels = vec![0_u8; (EDGE * EDGE) as usize];
    for (label, left, top, width, height) in blocks {
        for y in *top..top + height {
            for x in *left..left + width {
                labels[(y * EDGE + x) as usize] = *label;
            }
        }
    }
    Decomposition {
        labels,
        count,
        width: EDGE,
        height: EDGE,
    }
}

fn at(decomposition: &Decomposition, x: u32, y: u32) -> u8 {
    decomposition.labels[(y * EDGE + x) as usize]
}

#[test]
fn a_stray_fragment_leaves_its_layer() {
    let refined = refine(decomposition(1, &[(1, 10, 10, 40, 40), (1, 90, 90, 2, 2)]));
    assert_eq!(at(&refined, 20, 20), 1, "the subject stays");
    assert_eq!(
        at(&refined, 90, 90),
        0,
        "the speck goes back to the backdrop"
    );
}

#[test]
fn a_part_of_comparable_size_stays_with_its_layer() {
    // Two hands of one subject the body hides between them are one layer, not a stray.
    let refined = refine(decomposition(
        1,
        &[(1, 10, 10, 30, 30), (1, 60, 10, 20, 20)],
    ));
    assert_eq!(at(&refined, 70, 20), 1);
}

#[test]
fn a_pinhole_is_filled_but_a_window_is_not() {
    let mut canvas = decomposition(1, &[(1, 10, 10, 60, 60)]);
    for (x, y) in [(30, 30), (31, 30)] {
        canvas.labels[(y * EDGE + x) as usize] = 0;
    }
    for y in 45..60 {
        for x in 45..60 {
            canvas.labels[(y * EDGE + x) as usize] = 0;
        }
    }
    let refined = refine(canvas);
    assert_eq!(at(&refined, 30, 30), 1, "two pixels are a pinhole");
    assert_eq!(
        at(&refined, 50, 50),
        0,
        "the backdrop shows through a real window"
    );
}

#[test]
fn a_hole_another_layer_shows_through_is_not_filled() {
    let refined = refine(decomposition(2, &[(1, 10, 10, 60, 60), (2, 30, 30, 2, 2)]));
    assert_eq!(at(&refined, 30, 30), 2, "the nearer layer keeps its pixels");
    assert_eq!(refined.count, 2);
}

#[test]
fn layers_left_without_pixels_are_removed_in_stack_order() {
    let refined = refine(decomposition(3, &[(1, 0, 0, 40, 40), (3, 60, 60, 40, 40)]));
    assert_eq!(refined.count, 2);
    assert_eq!(at(&refined, 5, 5), 1);
    assert_eq!(
        at(&refined, 70, 70),
        2,
        "the third layer moves down into the empty slot"
    );
}

#[test]
fn the_salient_layer_is_soft_only_at_its_edge() {
    let canvas = decomposition(2, &[(1, 10, 10, 40, 40), (2, 60, 60, 20, 20)]);
    let saliency = canvas
        .labels
        .iter()
        .map(|label| if *label == 1 { 40 } else { 0 })
        .collect::<Vec<_>>();
    let (owner, alpha) = salient_edge(&canvas, &saliency).expect("owner");
    assert_eq!(owner, 1);
    let alpha_at = |x: u32, y: u32| alpha[(y * EDGE + x) as usize];
    assert_eq!(
        alpha_at(30, 30),
        255,
        "a weakly salient inside is still solid"
    );
    assert_eq!(alpha_at(10, 30), 40, "the edge takes saliency's coverage");
    assert_eq!(alpha_at(70, 70), 0, "other layers are not this layer's");
    assert!(salient_edge(&canvas, &[1, 2]).is_none(), "shape must match");
}

#[test]
fn completion_covers_each_subject_and_its_rim() {
    let holes = backdrop_holes(&decomposition(1, &[(1, 40, 40, 10, 10)]));
    let hole = |x: u32, y: u32| holes[(y * EDGE + x) as usize];
    assert!(hole(45, 45));
    assert!(hole(37, 45), "the rim three pixels out is completed too");
    assert!(!hole(36, 45));
}
