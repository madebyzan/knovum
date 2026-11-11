use super::super::{output::CanonicalImage, source::SourceImage};
use crate::{
    foundation::app_error::AppError,
    local_resources::{model_catalog::LocalModelId, preparation},
    spaces::derivation::image_tensors,
};
use sha2::{Digest, Sha256};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

pub(super) async fn execute(
    app: &AppHandle,
    model: LocalModelId,
    source: &SourceImage,
    owner_id: &str,
    cancellation: &CancellationToken,
) -> Result<CanonicalImage, AppError> {
    let resources = preparation::prepare_models_for_job(
        app,
        model.preparation(),
        owner_id,
        "Remove image background",
        cancellation,
        &mut |_| {},
    )
    .await?;
    let identity = image_tensors::segmentation_identity(
        model,
        owner_id,
        format!("sha256:{}", hex::encode(Sha256::digest(&source.bytes))),
    )?;
    let image = crate::images::remove_background(
        app,
        source.bytes.clone(),
        &identity,
        &resources,
        cancellation,
    )
    .await?;
    Ok(CanonicalImage {
        digest: format!("sha256:{}", hex::encode(Sha256::digest(&image.png))),
        png: image.png,
        width: image.width,
        height: image.height,
    })
}
