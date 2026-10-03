use image::{DynamicImage, Rgba, RgbaImage};

use super::pixels::{apply_mask, model_input, MODEL_PIXELS};

#[test]
fn model_input_is_channel_first_and_normalized() {
    let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([255, 128, 0, 255])));
    let input = model_input(&image);
    assert_eq!(input.len(), MODEL_PIXELS * 3);
    assert!((input[0] - 127.0 / 256.0).abs() < f32::EPSILON);
    assert_eq!(input[MODEL_PIXELS], 0.0);
    assert_eq!(input[MODEL_PIXELS * 2], -0.5);
}

#[test]
fn mask_replaces_source_alpha_at_original_resolution() {
    let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(2, 3, Rgba([12, 34, 56, 200])));
    let foreground = apply_mask(image, &vec![0.25; MODEL_PIXELS]).unwrap();
    assert_eq!(foreground.dimensions(), (2, 3));
    assert_eq!(foreground.get_pixel(0, 0), &Rgba([12, 34, 56, 64]));
}
