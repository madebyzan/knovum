//! Slices a sticker sheet — a grid of stickers drawn over one plain backdrop, each sticker a
//! character with the caption that goes with it — into one sticker per cell, framed as a WeChat
//! sticker album main image. This is a reading of the sheet's geometry, not of its depth: every
//! sticker stands beside the others rather than in front of them, so no model is involved and the
//! caption under a character stays with it.
//!
//! 1. the backdrop the sheet edge reaches is lifted off ([`backdrop`]);
//! 2. the sheet's grid is read from its foreground profile ([`grid`]);
//! 3. each cell is framed to the album's main-image standard ([`sticker`]).
//!
//! How many stickers the sheet carries is the sheet's own, so the set is published through
//! [`image_set`].
//!
//! [`image_set`]: super::image_set

mod backdrop;
mod grid;
mod sticker;

use image::RgbaImage;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use super::{
    action_execution::ExecutionAuthority,
    image_set::{self, ImageSet, ImageSetRequest},
    image_tensors, CompletedDerivationOutput,
};
use crate::foundation::app_error::AppError;

pub(crate) static SET: ImageSet = ImageSet {
    action_kind: "slice-sticker-sheet",
    engine: "local:sticker-sheet",
    lineage: "Sticker cut from source sticker sheet",
    member: "sticker",
    // A sheet is several stickers; one sticker is already a sticker.
    members: 2..=(grid::MAX_SPLITS * grid::MAX_SPLITS) as usize,
    unsupported: "This image is not a sheet of several stickers",
};

pub(crate) async fn slice_for_action(
    app: &AppHandle,
    request: ImageSetRequest,
    authority: &ExecutionAuthority,
    cancellation: &CancellationToken,
) -> Result<Vec<CompletedDerivationOutput>, AppError> {
    image_set::derive(app, &SET, request, authority, cancellation, |png| {
        let token = cancellation.clone();
        async move {
            crate::foundation::blocking::run(move || slice(&png, &token))
                .await
                .map_err(AppError::from)
        }
    })
    .await
}

/// The sheet's stickers as album main-image PNGs, in reading order.
fn slice(png: &[u8], cancellation: &CancellationToken) -> Result<Vec<Vec<u8>>, String> {
    let sheet = backdrop::lift(&image_tensors::decode(png)?)?;
    frame_stickers(&sheet)
        .iter()
        .map(|sticker| {
            if cancellation.is_cancelled() {
                return Err("Sticker sheet slicing was cancelled".to_owned());
            }
            sticker::encode(sticker)
        })
        .collect()
}

fn frame_stickers(sheet: &RgbaImage) -> Vec<RgbaImage> {
    let drawn = sheet.pixels().map(|pixel| pixel[3] > 0).collect::<Vec<_>>();
    grid::cells(&drawn, sheet.dimensions())
        .into_iter()
        .map(|bounds| sticker::frame(sheet, bounds))
        .collect()
}

#[cfg(test)]
mod tests;
