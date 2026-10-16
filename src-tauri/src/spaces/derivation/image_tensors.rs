//! Tensor preparation shared by the image derivations. The layouts and normalisation constants
//! below belong to the ONNX models themselves, so both the single-output image operations and
//! layer splitting read them from here instead of restating them per feature.

use crate::ai::{
    contracts::image_wire::TensorDataType, local_runtime::image::compute::ImageTensor,
};
use image::{
    imageops::{self, FilterType},
    DynamicImage, ImageFormat, ImageReader, Limits, RgbaImage,
};
use std::io::Cursor;

/// EfficientSAM admits four megapixels, which is one 2048 pixel long edge.
const SELECTION_LONG_EDGE: u32 = 2048;
/// Depth-Anything expects a 518 pixel short edge aligned to its patch size of 14.
const DEPTH_EDGE: f64 = 518.0;
const DEPTH_PATCH: f64 = 14.0;

pub(super) fn decode(bytes: &[u8]) -> Result<RgbaImage, String> {
    let mut reader = ImageReader::with_format(Cursor::new(bytes), ImageFormat::Png);
    let mut limits = Limits::default();
    limits.max_image_width = Some(16384);
    limits.max_image_height = Some(16384);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    let source = reader
        .decode()
        .map_err(|error| error.to_string())?
        .to_rgba8();
    if u64::from(source.width()) * u64::from(source.height()) > 16_777_216 {
        return Err("Local image source exceeds 16 megapixels".into());
    }
    Ok(source)
}

/// Planar `1 × 3 × height × width` RGB bytes, which is how every image model takes its source.
pub(super) fn rgb_tensor(image: &RgbaImage) -> ImageTensor {
    let mut bytes = Vec::with_capacity(image.width() as usize * image.height() as usize * 3);
    for channel in 0..3 {
        bytes.extend(image.pixels().map(|pixel| pixel[channel]));
    }
    ImageTensor {
        bytes,
        shape: [1, 3, image.height(), image.width()],
        data_type: TensorDataType::Uint8,
    }
}

/// Reduces the source to the selection budget before building its RGB tensor.
pub(super) fn selection_input(original: &RgbaImage) -> ImageTensor {
    if original.width().max(original.height()) > SELECTION_LONG_EDGE {
        let reduced = DynamicImage::ImageRgba8(original.clone())
            .resize(
                SELECTION_LONG_EDGE,
                SELECTION_LONG_EDGE,
                FilterType::Lanczos3,
            )
            .to_rgba8();
        rgb_tensor(&reduced)
    } else {
        rgb_tensor(original)
    }
}

pub(super) fn depth_tensor(source: &RgbaImage) -> Result<ImageTensor, String> {
    let scale_h = DEPTH_EDGE / f64::from(source.height());
    let scale_w = DEPTH_EDGE / f64::from(source.width());
    let scale = if (1.0 - scale_h).abs() < (1.0 - scale_w).abs() {
        scale_h
    } else {
        scale_w
    };
    let width = patch_aligned(f64::from(source.width()) * scale);
    let height = patch_aligned(f64::from(source.height()) * scale);
    if u64::from(width) * u64::from(height) > 1_048_576 {
        return Err("Image aspect ratio exceeds the depth processor budget".into());
    }
    let resized = imageops::resize(source, width, height, FilterType::CatmullRom);
    let mut bytes = Vec::with_capacity(width as usize * height as usize * 12);
    for (channel, (mean, deviation)) in [(0.485, 0.229), (0.456, 0.224), (0.406, 0.225)]
        .into_iter()
        .enumerate()
    {
        for pixel in resized.pixels() {
            bytes.extend_from_slice(
                &((f32::from(pixel[channel]) / 255.0 - mean) / deviation).to_le_bytes(),
            );
        }
    }
    Ok(ImageTensor {
        bytes,
        shape: [1, 3, height, width],
        data_type: TensorDataType::Float32LittleEndian,
    })
}

fn patch_aligned(edge: f64) -> u32 {
    (((edge / DEPTH_PATCH).round_ties_even() as u32) * DEPTH_PATCH as u32).max(DEPTH_PATCH as u32)
}
