//! Discovery for photographic sources: saliency proposes where subjects are, EfficientSAM turns
//! the proposals into masks, depth orders them far to near, and inpainting completes the backdrop.

mod ordering;
mod passes;

use super::{seeds, Layers};
use crate::foundation::app_error::AppError;
use image::RgbaImage;
use sha2::{Digest, Sha256};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

pub(super) async fn decompose(
    app: &AppHandle,
    source: &RgbaImage,
    owner_id: &str,
    max_layers: u8,
    cancellation: &CancellationToken,
) -> Result<Layers, AppError> {
    let revision = format!("sha256:{}", hex::encode(Sha256::digest(source.as_raw())));
    let identity = (owner_id, revision.as_str());
    let saliency = passes::saliency(app, source, identity, cancellation).await?;
    let seeds = seeds::from_saliency(&saliency, (source.width(), source.height()));
    let ranks = passes::discover(app, source, identity, (seeds, max_layers), cancellation).await?;
    let depth = passes::depth(app, source, identity, cancellation).await?;
    let decomposition = ordering::stack(ranks, (source.width(), source.height()), &depth)?;
    if decomposition.count == 0 {
        return Err("No layers were discovered in this image".into());
    }
    let mask = decomposition
        .labels
        .iter()
        .map(|label| if *label == 0 { 0 } else { 255 })
        .collect::<Vec<_>>();
    // A source covered end to end by subjects has no backdrop to complete, and inpainting that
    // much of the canvas is not worth its cost either; the backdrop layer then keeps the holes.
    let backdrop = match mask.iter().filter(|value| **value == 255).count() * 4 < mask.len() * 3 {
        true => passes::complete(app, source, mask, identity, cancellation)
            .await
            .ok(),
        false => None,
    };
    Ok(Layers {
        alpha: ordering::salient_layer(&decomposition, saliency),
        backdrop,
        decomposition,
    })
}
