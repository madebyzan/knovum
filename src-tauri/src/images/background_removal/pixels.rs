//! U²-Net family (`u2netp`, `u2net`, `u2net_human_seg`, `silueta`) pre- and post-processing,
//! transcribed from rembg `BaseSession.normalize`, `U2netSession.predict` and `naive_cutout`.
//! The worker owns min-max normalization of the graph output; this side owns geometry, the
//! uint8 quantization of the normalized field and alpha composition.

use image::{imageops::FilterType, DynamicImage, GrayImage, RgbaImage};

use crate::ai::contracts::image_spec::{
    Geometry, Scale, Standardization, U2NET_RESOLUTION, U2NET_SPEC,
};

const EDGE: u32 = match U2NET_SPEC.geometry {
    Geometry::Stretch { width, .. } => width,
    _ => U2NET_RESOLUTION,
};
const PIXELS: usize = EDGE as usize * EDGE as usize;
const MEAN: [f32; 3] = match U2NET_SPEC.standardization {
    Standardization::ImageNet { mean, .. } => mean,
    _ => [0.485, 0.456, 0.406],
};
const STD: [f32; 3] = match U2NET_SPEC.standardization {
    Standardization::ImageNet { std, .. } => std,
    _ => [0.229, 0.224, 0.225],
};
/// Lower bound of the whole-image scale, as in the reference (`max(np.max(im_ary), 1e-6)`).
const MIN_SCALE: f32 = match U2NET_SPEC.scale {
    Scale::AdaptiveMaxScale { min_scale } => min_scale,
    _ => 1e-6,
};

/// Channel-first ImageNet-standardized tensor. The source is stretched to the model square with
/// Lanczos (no letterbox), then divided by the single brightest channel value of the resized image
/// before standardization — not by 255 — so dark sources reach the range the model was trained on.
pub fn model_input(image: &DynamicImage) -> Vec<f32> {
    let resized = image::imageops::resize(&image.to_rgb8(), EDGE, EDGE, FilterType::Lanczos3);
    let brightest = resized.as_raw().iter().copied().max().unwrap_or(0);
    let scale = f32::from(brightest).max(MIN_SCALE);
    let mut input = vec![0.0; PIXELS * 3];
    for (index, pixel) in resized.pixels().enumerate() {
        for channel in 0..3 {
            let value = f32::from(pixel[channel]) / scale;
            input[index + PIXELS * channel] = (value - MEAN[channel]) / STD[channel];
        }
    }
    input
}

/// The worker's normalized `[0, 1]` field as a grayscale mask at `width`×`height`. Quantization
/// truncates like the reference's `astype("uint8")`; the upsample is Lanczos. A value outside
/// `[0, 1]` means the worker broke its contract and is rejected rather than clamped.
pub fn mask_at(field: &[f32], width: u32, height: u32) -> Result<GrayImage, String> {
    if field.len() != PIXELS {
        return Err("invalid_output: segmentation mask has invalid dimensions".into());
    }
    if field.iter().any(|value| !(0.0..=1.0).contains(value)) {
        return Err("invalid_output: segmentation mask is not normalized".into());
    }
    let bytes = field.iter().map(|value| (value * 255.0) as u8).collect();
    let model_mask =
        GrayImage::from_raw(EDGE, EDGE, bytes).ok_or("invalid_output: segmentation mask")?;
    Ok(image::imageops::resize(
        &model_mask,
        width,
        height,
        FilterType::Lanczos3,
    ))
}

/// Cuts the foreground out. Source alpha is multiplied by the mask (`Image.composite` semantics),
/// so an already transparent pixel stays transparent. Colour channels keep the source values: a
/// straight-alpha PNG does not take the reference's premultiplied RGB, which darkens soft edges.
pub fn apply_mask(image: DynamicImage, field: &[f32]) -> Result<RgbaImage, String> {
    let mask = mask_at(field, image.width(), image.height())?;
    let mut foreground = image.to_rgba8();
    for (pixel, alpha) in foreground.pixels_mut().zip(mask.pixels()) {
        pixel[3] = multiply(pixel[3], alpha[0]);
    }
    Ok(foreground)
}

/// `round(a * b / 255)`, the rounding Pillow's paste blend uses.
fn multiply(a: u8, b: u8) -> u8 {
    ((u16::from(a) * u16::from(b) + 127) / 255) as u8
}
