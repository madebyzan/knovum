use image::{DynamicImage, Rgba, RgbaImage};

use super::pixels::{apply_mask, mask_at, model_input, model_input_for_spec};
use crate::ai::contracts::image_spec::ISNET_DIS_SPEC;

const PIXELS: usize = 1024 * 1024;

fn standardized(value: f32, channel: usize) -> f32 {
    const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
    const STD: [f32; 3] = [0.229, 0.224, 0.225];
    (value - MEAN[channel]) / STD[channel]
}

#[test]
fn model_input_is_channel_first_and_imagenet_standardized() {
    let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([255, 128, 0, 255])));
    let input = model_input(&image);
    assert_eq!(input.len(), PIXELS * 3);
    assert!((input[0] - standardized(1.0, 0)).abs() < 1e-5);
    assert!((input[PIXELS] - standardized(128.0 / 255.0, 1)).abs() < 1e-5);
    assert!((input[PIXELS * 2] - standardized(0.0, 2)).abs() < 1e-5);
}

#[test]
fn model_input_scales_by_the_brightest_value_of_the_whole_image() {
    let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([120, 60, 0, 255])));
    let (input, _) = model_input_for_spec(&image, &ISNET_DIS_SPEC);
    // IS-Net standardization mean: [0.5, 0.5, 0.5], std: [1.0, 1.0, 1.0]
    assert!((input[0] - (1.0 - 0.5) / 1.0).abs() < 1e-5);
    assert!((input[PIXELS] - (0.5 - 0.5) / 1.0).abs() < 1e-5);
    let black = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([0, 0, 0, 255])));
    let (black_input, _) = model_input_for_spec(&black, &ISNET_DIS_SPEC);
    assert!(black_input.iter().all(|value| value.is_finite()));
}

#[test]
fn mask_multiplies_source_alpha_at_original_resolution() {
    let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(2, 3, Rgba([12, 34, 56, 200])));
    let foreground = apply_mask(image, &vec![0.5; PIXELS]).unwrap();
    assert_eq!(foreground.dimensions(), (2, 3));
    // 0.5 truncates to 127, and 200 * 127 / 255 rounds to 100.
    assert_eq!(foreground.get_pixel(0, 0), &Rgba([12, 34, 56, 100]));
}

#[test]
fn mask_rejects_a_field_the_worker_did_not_normalize() {
    assert!(mask_at(&vec![1.5; PIXELS], 2, 2).is_err());
    assert!(mask_at(&vec![0.5; PIXELS - 1], 2, 2).is_err());
}
