//! Local layer splitting. One rendered source becomes a stack of full-canvas RGBA layers whose
//! composite reproduces the source. Discovery has two paths, chosen by measured input structure:
//! sources drawn over a uniform backdrop are read from pixel connectivity, photographic sources
//! are read with the local image models. Each path is complete on its own.

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
    /// The backdrop with every subject removed. Absent when no pass could complete it, in which
    /// case the backdrop layer keeps the holes the subjects left.
    backdrop: Option<RgbaImage>,
}

impl Layers {
    pub(super) fn count(&self) -> u8 {
        self.decomposition.count
    }
}

/// Splits a rendered PNG into layer PNGs, backdrop first. The returned length is the discovered
/// layer count plus the backdrop.
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
