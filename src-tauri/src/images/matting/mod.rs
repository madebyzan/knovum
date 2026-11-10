//! High-resolution alpha matting, guided edge refinement, and color decontamination.
//!
//! Provides sub-pixel edge alignment for neural segmentation masks and suppresses
//! background color halos on soft boundaries (such as fine hair, fur, and transparent objects).

pub mod decontaminate;
pub mod guided_filter;

use image::{DynamicImage, GrayImage, RgbaImage};

const DEFAULT_RADIUS: usize = 8;
const DEFAULT_EPS: f32 = 1e-4;
const DEFAULT_SUBSAMPLE: usize = 2;

/// Refines a coarse segmentation mask using the source image luminance as edge-preserving guidance.
pub fn refine_mask(guidance: &DynamicImage, coarse_mask: &GrayImage) -> GrayImage {
    let (width, height) = (guidance.width() as usize, guidance.height() as usize);
    if width == 0
        || height == 0
        || coarse_mask.width() as usize != width
        || coarse_mask.height() as usize != height
    {
        return coarse_mask.clone();
    }

    let rgb = guidance.to_rgb8();
    let luma_guidance: Vec<f32> = rgb
        .pixels()
        .map(|p| {
            (0.299 * f32::from(p[0]) + 0.587 * f32::from(p[1]) + 0.114 * f32::from(p[2])) / 255.0
        })
        .collect();

    let float_mask: Vec<f32> = coarse_mask
        .pixels()
        .map(|p| f32::from(p[0]) / 255.0)
        .collect();

    let refined_floats = guided_filter::fast_guided_filter(
        &luma_guidance,
        &float_mask,
        width,
        height,
        DEFAULT_RADIUS,
        DEFAULT_EPS,
        DEFAULT_SUBSAMPLE,
    );

    let refined_bytes: Vec<u8> = refined_floats
        .into_iter()
        .map(|v| (v * 255.0 + 0.5).clamp(0.0, 255.0) as u8)
        .collect();

    GrayImage::from_raw(width as u32, height as u32, refined_bytes)
        .unwrap_or_else(|| coarse_mask.clone())
}

/// Refines the mask and performs edge color decontamination on the source RGBA image.
pub fn refine_cutout(source: &DynamicImage, coarse_mask: &GrayImage) -> RgbaImage {
    let refined_alpha = refine_mask(source, coarse_mask);
    let decontaminated =
        decontaminate::decontaminate_edges(&source.to_rgba8(), refined_alpha.as_raw());

    let mut output = decontaminated;
    for (pixel, alpha) in output.pixels_mut().zip(refined_alpha.pixels()) {
        pixel[3] = ((u16::from(pixel[3]) * u16::from(alpha[0]) + 127) / 255) as u8;
    }
    output
}
