//! Segmentation pre- and post-processing for the 1024×1024 high-resolution models, feeding the
//! native alpha matting and defringing in `matting`.

use image::{imageops::FilterType, DynamicImage, GrayImage, RgbaImage};

use crate::{
    ai::contracts::image_spec::{Geometry, ImageModelSpec, Resample, Scale, Standardization},
    local_resources::model_catalog::LocalModelId,
};

/// Builds the standardized input tensor according to the model's frozen specification, returning
/// the tensor and the model's square input edge.
pub fn model_input_for_spec(image: &DynamicImage, spec: &ImageModelSpec) -> (Vec<f32>, u32) {
    let edge = match spec.geometry {
        Geometry::Stretch { width, height } if width == height => width,
        _ => panic!("frozen segmentation geometry"),
    };
    let pixels = edge as usize * edge as usize;
    let (mean, std) = match spec.standardization {
        Standardization::ImageNet { mean, std } => (mean, std),
        _ => panic!("frozen segmentation standardization"),
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
        Scale::Identity => panic!("segmentation specs never pass raw pixels"),
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

/// Prepares the input tensor for the segmentation model identified by `spec_id`, returning the
/// tensor and the model's square mask edge.
pub fn model_input_for_spec_id(
    image: &DynamicImage,
    spec_id: &str,
) -> Result<(Vec<f32>, u32), String> {
    let model = LocalModelId::from_image_segmentation_spec(spec_id)?;
    Ok(model_input_for_spec(image, model.segmentation_spec()?))
}

/// Default input preparation using BiRefNet frozen specification for unit tests.
#[cfg(test)]
pub fn model_input(image: &DynamicImage) -> Vec<f32> {
    model_input_for_spec(image, &crate::ai::contracts::image_spec::BIREFNET_SPEC).0
}

/// The worker's normalized `[0, 1]` field, taken at the model's square `edge` from its frozen
/// specification, as a grayscale mask at `width`×`height`.
pub fn mask_at(field: &[f32], edge: u32, width: u32, height: u32) -> Result<GrayImage, String> {
    if field.len() != edge as usize * edge as usize {
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
pub fn apply_mask(image: DynamicImage, field: &[f32], edge: u32) -> Result<RgbaImage, String> {
    let coarse_mask = mask_at(field, edge, image.width(), image.height())?;
    Ok(crate::images::matting::refine_cutout(&image, &coarse_mask))
}
