use super::super::{contracts::ImageSelection, source::SourceImage};
use crate::{
    ai::{
        contracts::image_wire::TensorDataType,
        local_runtime::image::compute::{ImageTensor, ImageTensorOperation},
    },
    foundation::app_error::AppError,
    local_resources::model_catalog::{LocalModelCapability, LocalModelId},
    spaces::derivation::{image_tensors, regions},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{imageops::FilterType, DynamicImage, ImageFormat, RgbaImage};
use std::io::Cursor;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

/// Coverage at or above this is inside the selection: the decoder's own decision boundary.
const INSIDE: u8 = 128;

/// How much of every source pixel the selection covers, as `[0, 255]` at the source's resolution:
/// the decoder's soft edge, with brushed strokes added in full.
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
    let (original, input) =
        crate::foundation::blocking::run(move || prepare_source(&bytes)).await?;
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
    crate::foundation::blocking::run(move || {
        let (width, height) = original.dimensions();
        if tensor.shape != [1, 1, height, width] || tensor.data_type != TensorDataType::Uint8 {
            return Err("Invalid selected-region mask".to_owned());
        }
        let mut coverage = tensor.bytes;
        if !selection.strokes.is_empty() {
            let brush = super::super::mask::rasterize(width, height, &selection.strokes, &token)?;
            for (value, brush) in coverage.iter_mut().zip(brush.pixels()) {
                *value = (*value).max(brush[0]);
            }
        }
        if !coverage.iter().any(|value| *value >= INSIDE) {
            return Err("No foreground was selected; adjust the prompts".to_owned());
        }
        Ok((
            original,
            ImageTensor {
                bytes: coverage,
                shape: [1, 1, height, width],
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
    crate::foundation::blocking::run(move || {
        let field = image::GrayImage::from_raw(mask.shape[3], mask.shape[2], mask.bytes)
            .ok_or("Invalid selected-region mask")?;
        let mut png = Vec::new();
        let image = DynamicImage::ImageLuma8(field);
        let preview = if image.width().max(image.height()) > 2048 {
            image.resize(2048, 2048, FilterType::Triangle)
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

/// The pixels removing the selection repaints, as a binary mask: everything inside it, grown so the
/// subject's rim and contact shadow go with it.
pub(super) fn removal_hole(selection: ImageTensor) -> ImageTensor {
    let [_, _, height, width] = selection.shape;
    let inside = selection
        .bytes
        .iter()
        .map(|value| *value >= INSIDE)
        .collect::<Vec<_>>();
    ImageTensor {
        bytes: regions::removal_holes(&inside, (width, height))
            .into_iter()
            .map(|hole| if hole { 255 } else { 0 })
            .collect(),
        ..selection
    }
}

pub(super) fn prepare_source(bytes: &[u8]) -> Result<(RgbaImage, ImageTensor), String> {
    let original = image_tensors::decode(bytes)?;
    let input = image_tensors::rgb_tensor(&original);
    Ok((original, input))
}
