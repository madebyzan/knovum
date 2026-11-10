//! Segmentation pre- and post-processing, supporting both 320x320 legacy models
//! and 1024x1024 high-resolution models with native alpha matting and defringing.

use image::{imageops::FilterType, DynamicImage, GrayImage, RgbaImage};

use crate::ai::contracts::image_spec::{
    Geometry, ImageModelSpec, Resample, Scale, Standardization, BIREFNET_PORTRAIT_SPEC,
    BIREFNET_SPEC, HIGH_RES_SEGMENTATION_RESOLUTION, ISNET_ANIME_SPEC, ISNET_DIS_SPEC, RMBG14_SPEC,
    RMBG20_SPEC,
};

/// Builds the standardized input tensor according to the model's frozen specification.
pub fn model_input_for_spec(image: &DynamicImage, spec: &ImageModelSpec) -> (Vec<f32>, u32) {
    let edge = match spec.geometry {
        Geometry::Stretch { width, .. } => width,
        _ => HIGH_RES_SEGMENTATION_RESOLUTION,
    };
    let pixels = edge as usize * edge as usize;
    let (mean, std) = match spec.standardization {
        Standardization::ImageNet { mean, std } => (mean, std),
        _ => ([0.485, 0.456, 0.406], [0.229, 0.224, 0.225]),
    };
    let filter = match spec.resample {
        Resample::Lanczos3 => FilterType::Lanczos3,
        Resample::Triangle => FilterType::Triangle,
        Resample::CatmullRom => FilterType::CatmullRom,
        Resample::Nearest => FilterType::Nearest,
    };
    let resized = image::imageops::resize(&image.to_rgb8(), edge, edge, filter);
    let scale = match spec.scale {
        Scale::AdaptiveMaxScale { min_scale } => {
            let brightest = resized.as_raw().iter().copied().max().unwrap_or(0);
            f32::from(brightest).max(min_scale)
        }
        Scale::UnitScale => 255.0,
        Scale::Identity => 1.0,
    };
    let mut input = vec![0.0; pixels * 3];
    for (index, pixel) in resized.pixels().enumerate() {
        for channel in 0..3 {
            let value = f32::from(pixel[channel]) / scale;
            input[index + pixels * channel] = (value - mean[channel]) / std[channel];
        }
    }
    (input, edge)
}

/// Prepares the input tensor based on the model specification ID.
pub fn model_input_for_spec_id(image: &DynamicImage, spec_id: &str) -> Result<Vec<f32>, String> {
    let spec = match spec_id {
        crate::local_resources::model_catalog::IMAGE_SEGMENTATION_BIREFNET_SPEC_ID => {
            &BIREFNET_SPEC
        }
        crate::local_resources::model_catalog::IMAGE_SEGMENTATION_RMBG20_SPEC_ID => &RMBG20_SPEC,
        crate::local_resources::model_catalog::IMAGE_SEGMENTATION_PORTRAIT_SPEC_ID => {
            &BIREFNET_PORTRAIT_SPEC
        }
        crate::local_resources::model_catalog::IMAGE_SEGMENTATION_RMBG14_SPEC_ID => &RMBG14_SPEC,
        crate::local_resources::model_catalog::IMAGE_SEGMENTATION_DIS_SPEC_ID => &ISNET_DIS_SPEC,
        crate::local_resources::model_catalog::IMAGE_SEGMENTATION_ANIME_SPEC_ID => {
            &ISNET_ANIME_SPEC
        }
        _ => return Err(format!("Unsupported segmentation model spec: {spec_id}")),
    };
    Ok(model_input_for_spec(image, spec).0)
}

/// Default input preparation using BiRefNet frozen specification for unit tests.
#[cfg(test)]
pub fn model_input(image: &DynamicImage) -> Vec<f32> {
    model_input_for_spec(image, &BIREFNET_SPEC).0
}

/// The worker's normalized `[0, 1]` field as a grayscale mask at `width`×`height`.
/// Supports both 320x320 and 1024x1024 models dynamically.
pub fn mask_at(field: &[f32], width: u32, height: u32) -> Result<GrayImage, String> {
    let field_len = field.len();
    let edge = (field_len as f64).sqrt().round() as u32;
    if (edge as usize * edge as usize) != field_len {
        return Err("invalid_output: segmentation mask has invalid dimensions".into());
    }
    if field.iter().any(|value| !(0.0..=1.0).contains(value)) {
        return Err("invalid_output: segmentation mask is not normalized".into());
    }
    let bytes = field.iter().map(|value| (value * 255.0) as u8).collect();
    let model_mask =
        GrayImage::from_raw(edge, edge, bytes).ok_or("invalid_output: segmentation mask")?;
    Ok(image::imageops::resize(
        &model_mask,
        width,
        height,
        FilterType::Lanczos3,
    ))
}

/// Cuts the foreground out using guided alpha matting refinement and edge color decontamination.
pub fn apply_mask(image: DynamicImage, field: &[f32]) -> Result<RgbaImage, String> {
    let coarse_mask = mask_at(field, image.width(), image.height())?;
    Ok(crate::images::matting::refine_cutout(&image, &coarse_mask))
}
