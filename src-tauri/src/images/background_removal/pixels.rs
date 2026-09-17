use image::{imageops::FilterType, DynamicImage, GrayImage, Luma, RgbaImage};

pub const MODEL_RESOLUTION: usize = 320;
pub const MODEL_PIXELS: usize = MODEL_RESOLUTION * MODEL_RESOLUTION;

pub fn model_input(image: &DynamicImage) -> Vec<f32> {
    let resized = image
        .resize_exact(
            MODEL_RESOLUTION as u32,
            MODEL_RESOLUTION as u32,
            FilterType::Triangle,
        )
        .to_rgb8();
    let mut input = vec![0.0; MODEL_PIXELS * 3];
    for (index, pixel) in resized.pixels().enumerate() {
        input[index] = (f32::from(pixel[0]) - 128.0) / 256.0;
        input[index + MODEL_PIXELS] = (f32::from(pixel[1]) - 128.0) / 256.0;
        input[index + MODEL_PIXELS * 2] = (f32::from(pixel[2]) - 128.0) / 256.0;
    }
    input
}

pub fn apply_mask(image: DynamicImage, mask: &[f32]) -> Result<RgbaImage, String> {
    if mask.len() != MODEL_PIXELS {
        return Err("Background-removal mask has invalid dimensions".into());
    }
    let model_mask =
        GrayImage::from_fn(MODEL_RESOLUTION as u32, MODEL_RESOLUTION as u32, |x, y| {
            let index = y as usize * MODEL_RESOLUTION + x as usize;
            Luma([(mask[index].clamp(0.0, 1.0) * 255.0).round() as u8])
        });
    let resized = image::imageops::resize(
        &model_mask,
        image.width(),
        image.height(),
        FilterType::Triangle,
    );
    let mut foreground = image.to_rgba8();
    for (pixel, alpha) in foreground.pixels_mut().zip(resized.pixels()) {
        pixel[3] = alpha[0];
    }
    Ok(foreground)
}
