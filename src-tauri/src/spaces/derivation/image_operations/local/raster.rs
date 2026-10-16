use super::super::{contracts::ImageOperation, output::CanonicalImage};
use crate::{
    ai::{
        contracts::image_wire::TensorDataType,
        local_runtime::image::compute::{ImageTensor, ImageTensorOperation},
    },
    spaces::derivation::image_tensors,
};
use image::{
    imageops::{self, FilterType},
    DynamicImage, ImageFormat, RgbaImage,
};
use sha2::{Digest, Sha256};
use std::io::Cursor;
use tokio_util::sync::CancellationToken;

pub(super) fn prepare(
    bytes: &[u8],
    operation: &ImageOperation,
    cancellation: &CancellationToken,
) -> Result<(RgbaImage, ImageTensor, ImageTensorOperation), String> {
    let source = image_tensors::decode(bytes)?;
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
            (
                image_tensors::rgb_tensor(&source),
                ImageTensorOperation::Upscale,
            )
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
                image_tensors::rgb_tensor(&source),
                ImageTensorOperation::Inpaint(ImageTensor {
                    bytes: selected,
                    shape: [1, 1, source.height(), source.width()],
                    data_type: TensorDataType::Uint8,
                }),
            )
        }
        ImageOperation::GenerateDepth => (
            image_tensors::depth_tensor(&source)?,
            ImageTensorOperation::Depth,
        ),
        _ => return Err("Operation is not supported by local image inference".into()),
    };
    Ok((source, input, computation))
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
