use crate::images::{
    depth::relative,
    inpainting::{context_window, feather, feather_width, windows},
    upscaling::quantize,
};

#[test]
fn upscale_quantization_matches_numpy_ties_to_even_and_clipping() {
    assert_eq!(quantize(0.5 / 255.0).unwrap(), 0);
    assert_eq!(quantize(1.5 / 255.0).unwrap(), 2);
    assert_eq!(quantize(-1.0).unwrap(), 0);
    assert_eq!(quantize(2.0).unwrap(), 255);
    assert!(quantize(f32::NAN).is_err());
}

#[test]
fn depth_fields_reject_invalid_values_and_stretch_relative_extremes() {
    assert_eq!(relative(&[2.0, 4.0, 6.0]).unwrap(), [0.0, 0.5, 1.0]);
    assert!(relative(&[1.0, 1.0]).is_err());
    assert!(relative(&[1.0, f32::INFINITY]).is_err());
}

#[test]
fn a_small_hole_is_painted_at_source_resolution_inside_the_model_square() {
    // A 20 px hole in a large photograph: the window is the model's own square around it, so
    // nothing is resampled.
    assert_eq!(
        context_window([1000, 700, 1019, 719], (4000, 3000)),
        [754, 454, 1266, 966]
    );
}

#[test]
fn a_window_keeps_context_around_a_large_hole_and_stays_inside_the_source() {
    let [left, top, right, bottom] = context_window([100, 0, 899, 599], (1000, 800));
    assert_eq!(
        (left, right),
        (0, 1000),
        "wider than the source: the whole width"
    );
    assert_eq!(
        (top, bottom),
        (0, 800),
        "a hole at the top keeps its context below"
    );
    let [left, top, right, bottom] = context_window([0, 0, 9, 9], (300, 200));
    assert_eq!(
        [left, top, right, bottom],
        [0, 0, 300, 200],
        "a small source is read whole"
    );
}

fn mask_with(holes: &[[u32; 4]], (width, height): (u32, u32)) -> Vec<u8> {
    let mut mask = vec![0; (width * height) as usize];
    for [left, top, right, bottom] in holes {
        for y in *top..=*bottom {
            for x in *left..=*right {
                mask[(y * width + x) as usize] = 255;
            }
        }
    }
    mask
}

#[test]
fn holes_far_apart_are_painted_in_their_own_windows_at_source_scale() {
    let dimensions = (4000, 3000);
    let mask = mask_with(
        &[[100, 100, 119, 119], [3800, 2800, 3819, 2819]],
        dimensions,
    );
    let painted = windows(&mask, dimensions);
    assert_eq!(painted.len(), 2);
    for [left, top, right, bottom] in painted {
        assert_eq!(
            (right - left, bottom - top),
            (512, 512),
            "read at the model's own scale"
        );
    }
}

#[test]
fn holes_whose_context_overlaps_share_one_window() {
    let dimensions = (4000, 3000);
    let mask = mask_with(
        &[[1000, 1000, 1019, 1019], [1200, 1000, 1219, 1019]],
        dimensions,
    );
    let painted = windows(&mask, dimensions);
    assert_eq!(painted.len(), 1);
    let [left, _, right, _] = painted[0];
    assert!(
        left <= 1000 && right > 1219,
        "the shared window covers both holes"
    );
}

#[test]
fn the_fill_fades_out_just_beyond_the_hole() {
    // One hole pixel in the middle of a 7-pixel row, feathered over two pixels.
    let holes = [0, 0, 0, 255, 0, 0, 0];
    let weights = feather(&holes, (7, 1), 2);
    let expected = [0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0, 2.0 / 3.0, 1.0 / 3.0, 0.0];
    for (weight, expected) in weights.iter().zip(expected) {
        assert!((weight - expected).abs() < 1e-6, "{weights:?}");
    }
    assert_eq!(feather_width(512), 1, "unscaled windows blend one pixel");
    assert_eq!(
        feather_width(2048),
        4,
        "one pixel per source pixel a model pixel covers"
    );
}
