use super::super::{contracts::ImageOperation, output::CanonicalImage};
use crate::ai::{
    contracts::image_wire::TensorDataType,
    local_runtime::image::compute::{ImageTensor, ImageTensorOperation},
};
use image::{
    imageops::{self, FilterType},
    DynamicImage, ImageFormat, ImageReader, Limits, RgbaImage,
};
use sha2::{Digest, Sha256};
use std::io::Cursor;
use tokio_util::sync::CancellationToken;

pub(super) fn prepare(
    bytes: &[u8],
    operation: &ImageOperation,
    cancellation: &CancellationToken,
) -> Result<(RgbaImage, ImageTensor, ImageTensorOperation), String> {
    let source = decode(bytes)?;
    let (input, computation) = match operation {
        ImageOperation::Upscale { target_long_edge } => {
            if !matches!(target_long_edge, 2048 | 4096 | 8192)
                || source.width().max(source.height()) >= *target_long_edge
            {
                return Err("Upscale target must increase the rendered source resolution".into());
            }
            if u64::from(source.width()) * u64::from(source.height()) > 4_194_304
                || source.width().max(source.height()) > 4096
            {
                return Err(
                    "Upscale source exceeds the admitted 4 megapixel / 4096 pixel budget".into(),
                );
            }
            (rgb_tensor(&source), ImageTensorOperation::Upscale)
        }
        ImageOperation::RemoveObject { strokes } => {
            let mask = super::super::mask::rasterize(
                source.width(),
                source.height(),
                strokes,
                cancellation,
            )?;
            let selected = mask.pixels().map(|pixel| pixel[0]).collect();
            (
                rgb_tensor(&source),
                ImageTensorOperation::Inpaint(ImageTensor {
                    bytes: selected,
                    shape: [1, 1, source.height(), source.width()],
                    data_type: TensorDataType::Uint8,
                }),
            )
        }
        ImageOperation::GenerateDepth => (depth_tensor(&source)?, ImageTensorOperation::Depth),
        _ => return Err("Operation is not supported by local image inference".into()),
    };
    Ok((source, input, computation))
}

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

fn depth_tensor(source: &RgbaImage) -> Result<ImageTensor, String> {
    let scale_h = 518.0 / f64::from(source.height());
    let scale_w = 518.0 / f64::from(source.width());
    let scale = if (1.0 - scale_h).abs() < (1.0 - scale_w).abs() {
        scale_h
    } else {
        scale_w
    };
    let width = ((f64::from(source.width()) * scale / 14.0).round_ties_even() as u32 * 14).max(14);
    let height =
        ((f64::from(source.height()) * scale / 14.0).round_ties_even() as u32 * 14).max(14);
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

pub(super) fn materialize(
    mut source: RgbaImage,
    tensor: ImageTensor,
    operation: &ImageOperation,
    cancellation: &CancellationToken,
) -> Result<CanonicalImage, String> {
    let [_, channels, height, width] = tensor.shape;
    let image = match operation {
        ImageOperation::GenerateDepth => {
            let values = tensor
                .bytes
                .chunks_exact(4)
                .map(|bytes| {
                    let value =
                        f32::from_le_bytes(bytes.try_into().expect("Validated float tensor"));
                    (value.clamp(0.0, 1.0) * 65535.0).round() as u16
                })
                .collect();
            let field =
                image::ImageBuffer::<image::Luma<u16>, Vec<u16>>::from_raw(width, height, values)
                    .ok_or("Invalid depth field")?;
            DynamicImage::ImageLuma16(imageops::resize(
                &field,
                source.width(),
                source.height(),
                FilterType::CatmullRom,
            ))
        }
        ImageOperation::ExtractSelection { .. } => {
            let field = image::GrayImage::from_raw(width, height, tensor.bytes)
                .ok_or("Invalid selected-region mask")?;
            let selection =
                imageops::resize(&field, source.width(), source.height(), FilterType::Nearest);
            for (pixel, selected) in source.pixels_mut().zip(selection.pixels()) {
                pixel[3] = (u16::from(pixel[3]) * u16::from(selected[0]) / 255) as u8;
            }
            DynamicImage::ImageRgba8(source)
        }
        ImageOperation::Upscale { .. }
        | ImageOperation::RemoveObject { .. }
        | ImageOperation::RemoveSelectedObject { .. } => {
            if channels != 3 {
                return Err("Invalid RGB output tensor".into());
            }
            let count = (width * height) as usize;
            let alpha = imageops::resize(&source, width, height, FilterType::Lanczos3);
            let mut output = RgbaImage::new(width, height);
            for (index, pixel) in output.pixels_mut().enumerate() {
                if index % width as usize == 0 && cancellation.is_cancelled() {
                    return Err("Image operation was cancelled".into());
                }
                *pixel = image::Rgba([
                    tensor.bytes[index],
                    tensor.bytes[count + index],
                    tensor.bytes[count * 2 + index],
                    alpha.as_raw()[index * 4 + 3],
                ]);
            }
            let output = DynamicImage::ImageRgba8(output);
            if let ImageOperation::Upscale { target_long_edge } = operation {
                output.resize(*target_long_edge, *target_long_edge, FilterType::Lanczos3)
            } else {
                output
            }
        }
        _ => return Err("Unsupported local image output".into()),
    };
    if cancellation.is_cancelled() {
        return Err("Image operation was cancelled".into());
    }
    let (width, height) = (image.width(), image.height());
    let mut png = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    if png.len() > 64 * 1024 * 1024 {
        return Err("Generated PNG exceeds 64 MiB".into());
    }
    Ok(CanonicalImage {
        digest: format!("sha256:{}", hex::encode(Sha256::digest(&png))),
        png,
        width,
        height,
    })
}

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
