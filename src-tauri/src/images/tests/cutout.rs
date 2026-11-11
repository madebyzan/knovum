use image::{Rgba, RgbaImage};

use super::{box_mean, cut_out};

#[test]
fn box_means_are_clipped_to_the_image() {
    let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    // 3 × 2, radius 1: a corner averages its 2 × 2 neighbourhood, the middle its 3 × 2 one.
    let means = box_mean(&values, (3, 2), 1);
    assert!((means[0] - 3.0).abs() < 1e-6);
    assert!((means[1] - 3.5).abs() < 1e-6);
    assert_eq!(
        box_mean(&values, (3, 2), 0),
        values,
        "radius 0 is the identity"
    );
    let wide = box_mean(&values, (3, 2), 10);
    assert!(
        wide.iter().all(|mean| (mean - 3.5).abs() < 1e-6),
        "a window wider than the image"
    );
}

#[test]
fn coverage_becomes_alpha_and_keeps_existing_transparency() {
    let mut source = RgbaImage::from_pixel(4, 1, Rgba([10, 20, 30, 255]));
    source.put_pixel(3, 0, Rgba([10, 20, 30, 128]));
    let cut = cut_out(&source, &[1.0, 0.5, 0.0, 1.0]);
    let alphas = cut.pixels().map(|pixel| pixel[3]).collect::<Vec<_>>();
    assert_eq!(alphas, [255, 128, 0, 128]);
    assert_eq!(
        &cut.get_pixel(0, 0).0[..3],
        &[10, 20, 30],
        "an opaque pixel keeps its colour"
    );
}

#[test]
fn a_soft_edge_loses_the_backdrop_it_was_photographed_against() {
    // A red subject photographed against green, with a 4-pixel soft edge in the middle.
    let (width, height) = (256_u32, 8_u32);
    let coverage = (0..width * height)
        .map(|index| {
            let x = (index % width) as f32;
            ((130.0 - x) / 4.0).clamp(0.0, 1.0)
        })
        .collect::<Vec<_>>();
    let source = RgbaImage::from_fn(width, height, |x, y| {
        let alpha = coverage[(y * width + x) as usize];
        Rgba([
            (255.0 * alpha).round() as u8,
            (255.0 * (1.0 - alpha)).round() as u8,
            0,
            255,
        ])
    });
    let cut = cut_out(&source, &coverage);
    // Half way through the edge the photograph is half green; the cut-out is red again.
    let edge = cut.get_pixel(128, 4);
    assert_eq!(edge[3], 128);
    assert!(edge[0] > 240, "the subject's red is recovered: {edge:?}");
    assert!(edge[1] < 15, "the backdrop's green is gone: {edge:?}");
}
