//! Local layer splitting. One rendered source becomes a stack of RGBA layers, each cropped to the
//! pixels it carries so a layer is published at its own size. Discovery has two paths, chosen by
//! measured input structure: sources drawn over a uniform backdrop are read from pixel
//! connectivity, photographic sources are read with the local image models. Each path is complete
//! on its own.

mod components;
mod compose;
mod scene;
mod seeds;
mod tiled;

use crate::{foundation::app_error::AppError, spaces::derivation::image_tensors};
use image::RgbaImage;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

/// One label per source pixel: `0` is backdrop and `1..=count` are the discovered subjects,
/// ordered bottom to top.
pub(super) struct Decomposition {
    labels: Vec<u8>,
    count: u8,
    width: u32,
    height: u32,
}

pub(super) struct Layers {
    decomposition: Decomposition,
    /// Soft alpha for one label, when a pass produced a better edge than the label itself.
    alpha: Option<(u8, Vec<u8>)>,
    /// The backdrop as a layer of its own. Absent when the source was drawn over a flat fill,
    /// which carries no content a layer could hold, so the stack is then the subjects alone.
    backdrop: Option<RgbaImage>,
}

/// Splits a rendered PNG into layer PNGs: the backdrop first when it is a layer of its own, then
/// one layer per discovered subject. Layers that carry no content are left out, so the returned
/// length is what the caller publishes.
pub(super) async fn split(
    app: &AppHandle,
    png: Vec<u8>,
    owner_id: &str,
    max_layers: u8,
    cancellation: &CancellationToken,
) -> Result<Vec<Vec<u8>>, AppError> {
    let source = crate::foundation::blocking::run(move || image_tensors::decode(&png)).await?;
    let layers = match crate::foundation::blocking::run({
        let source = source.clone();
        move || Ok::<_, String>(tiled::detect(&source, max_layers))
    })
    .await?
    {
        Some(layers) => layers,
        None => scene::decompose(app, &source, owner_id, max_layers, cancellation).await?,
    };
    let token = cancellation.clone();
    crate::foundation::blocking::run(move || compose::encode(&source, &layers, &token))
        .await
        .map_err(Into::into)
}
