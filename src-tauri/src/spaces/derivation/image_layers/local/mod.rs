//! Local layer decomposition. One rendered source becomes a stack of RGBA layers, each cropped to
//! the pixels it carries so a layer is published at its own size:
//!
//! 1. saliency proposes where the subjects are and seeds the prompts ([`seeds`]);
//! 2. the selection model turns the prompts into disjoint subject masks;
//! 3. depth orders the subjects far to near ([`ordering`]);
//! 4. the masks are refined into layers — strays dropped, pinholes filled, the salient edge
//!    softened ([`matte`]);
//! 5. completion paints the backdrop the subjects covered.
//!
//! Every pass runs the model a person selected for its capability ([`passes`]).

mod compose;
mod matte;
mod ordering;
mod passes;
mod seeds;

use crate::{
    ai::contracts::image_compute::MAX_DISCOVERED_LAYERS, foundation::app_error::AppError,
    spaces::derivation::image_tensors,
};
use image::RgbaImage;
use sha2::{Digest, Sha256};
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
    /// What the subjects stood in front of. It is published only when it carries content of its
    /// own: the flat fill a graphic was drawn over, or nothing at all, is not a layer.
    backdrop: RgbaImage,
}

/// Completion continues the backdrop from what surrounds a hole; once the holes cover this share
/// of the source there is too little left to continue, and the backdrop keeps its holes instead.
const MAX_COMPLETED_SHARE: f32 = 0.75;

/// Splits a rendered PNG into layer PNGs: the backdrop first when it is a layer of its own, then
/// one layer per discovered subject. Layers that carry no content are left out, so the returned
/// length is what the caller publishes.
pub(super) async fn split(
    app: &AppHandle,
    png: Vec<u8>,
    owner_id: &str,
    cancellation: &CancellationToken,
) -> Result<Vec<Vec<u8>>, AppError> {
    let source = crate::foundation::blocking::run(move || image_tensors::decode(&png)).await?;
    let layers = decompose(app, &source, owner_id, cancellation).await?;
    let token = cancellation.clone();
    crate::foundation::blocking::run(move || compose::encode(&source, &layers, &token))
        .await
        .map_err(Into::into)
}

async fn decompose(
    app: &AppHandle,
    source: &RgbaImage,
    owner_id: &str,
    cancellation: &CancellationToken,
) -> Result<Layers, AppError> {
    let revision = format!("sha256:{}", hex::encode(Sha256::digest(source.as_raw())));
    let models = passes::Models::selected(app)?;
    let pass = passes::Pass {
        app,
        models: &models,
        owner_id,
        revision: &revision,
        cancellation,
    };
    let dimensions = (source.width(), source.height());
    let saliency = pass.saliency(source).await?;
    let seeds = seeds::from_saliency(&saliency, dimensions);
    let ranks = pass.discover(source, seeds, MAX_DISCOVERED_LAYERS).await?;
    let depth = pass.depth(source).await?;
    let (decomposition, alpha, holes) = crate::foundation::blocking::run(move || {
        let decomposition = matte::refine(ordering::stack(ranks, dimensions, &depth)?);
        let alpha = matte::salient_edge(&decomposition, &saliency);
        let holes = matte::backdrop_holes(&decomposition);
        Ok::<_, String>((decomposition, alpha, holes))
    })
    .await?;
    if decomposition.count == 0 {
        return Err("No layers were discovered in this image".into());
    }
    let covered = holes.iter().filter(|hole| **hole).count() as f32 / holes.len() as f32;
    let backdrop = if covered < MAX_COMPLETED_SHARE {
        let mask = holes
            .iter()
            .map(|hole| if *hole { 255 } else { 0 })
            .collect();
        pass.complete(source, mask).await?
    } else {
        compose::without_subjects(source, &holes)
    };
    Ok(Layers {
        decomposition,
        alpha,
        backdrop,
    })
}
