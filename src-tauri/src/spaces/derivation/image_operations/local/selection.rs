use super::super::{contracts::ImageSelection, source::SourceImage};
use crate::{
    ai::{
        contracts::image_wire::TensorDataType,
        local_runtime::image::compute::{ImageTensor, ImageTensorOperation},
    },
    app_error::AppError,
    local_resources::model_catalog::{LocalModelCapability, LocalModelId},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{
    imageops::{self, FilterType},
    DynamicImage, ImageFormat, RgbaImage,
};
use std::io::Cursor;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

pub(super) async fn compute(
    app: &AppHandle,
    model: LocalModelId,
    selection: &ImageSelection,
    source: &SourceImage,
    owner_id: &str,
    cancellation: &CancellationToken,
) -> Result<(RgbaImage, ImageTensor), AppError> {
    selection.validate()?;
    if model.capability() != LocalModelCapability::ImageSelection {
        return Err("Model is not an image selection model".into());
    }
    let bytes = source.bytes.clone();
    let (original, input) = crate::blocking::run(move || prepare_source(&bytes)).await?;
    let tensor = super::infer(
        app,
        model,
        source,
        owner_id,
        input,
        ImageTensorOperation::Select {
            points: selection.points.clone(),
            box_rect: selection.box_rect.clone(),
        },
        cancellation,
    )
    .await?;
    let selection = selection.clone();
    let token = cancellation.clone();
    crate::blocking::run(move || {
        let field = image::GrayImage::from_raw(tensor.shape[3], tensor.shape[2], tensor.bytes)
            .ok_or("Invalid selected-region mask")?;
        let mut field = imageops::resize(
            &field,
            original.width(),
            original.height(),
            FilterType::Nearest,
        );
        if !selection.strokes.is_empty() {
            let brush = super::super::mask::rasterize(
                original.width(),
                original.height(),
                &selection.strokes,
                &token,
            )?;
            for (mask, brush) in field.pixels_mut().zip(brush.pixels()) {
                mask[0] = mask[0].max(brush[0]);
            }
        }
        if !field.pixels().any(|pixel| pixel[0] != 0) {
            return Err("No foreground was selected; adjust the prompts".to_owned());
        }
        let shape = [1, 1, original.height(), original.width()];
        Ok((
            original,
            ImageTensor {
                bytes: field.into_raw(),
                shape,
                data_type: TensorDataType::Uint8,
            },
        ))
    })
    .await
    .map_err(Into::into)
}

pub(in super::super) async fn preview(
    app: &AppHandle,
    model: LocalModelId,
    selection: &ImageSelection,
    source: &SourceImage,
    owner_id: &str,
    cancellation: &CancellationToken,
) -> Result<String, AppError> {
    let (_, mask) = compute(app, model, selection, source, owner_id, cancellation).await?;
    crate::blocking::run(move || {
        let field = image::GrayImage::from_raw(mask.shape[3], mask.shape[2], mask.bytes)
            .ok_or("Invalid selected-region mask")?;
        let mut png = Vec::new();
        let image = DynamicImage::ImageLuma8(field);
        let preview = if image.width().max(image.height()) > 2048 {
            image.resize(2048, 2048, FilterType::Nearest)
        } else {
            image
        };
        preview
            .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
            .map_err(|error| error.to_string())?;
        Ok::<_, String>(STANDARD.encode(png))
    })
    .await
    .map_err(Into::into)
}

pub(super) fn prepare_source(bytes: &[u8]) -> Result<(RgbaImage, ImageTensor), String> {
    let original = super::raster::decode(bytes)?;
    let input = if original.width().max(original.height()) > 2048 {
        let reduced = DynamicImage::ImageRgba8(original.clone())
            .resize(2048, 2048, FilterType::Lanczos3)
            .to_rgba8();
        super::raster::rgb_tensor(&reduced)
    } else {
        super::raster::rgb_tensor(&original)
    };
    Ok((original, input))
}
