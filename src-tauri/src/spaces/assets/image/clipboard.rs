use std::fs;

use image::ImageFormat;
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::validation;
use crate::spaces::assets::asset_directory;

const MAX_CLIPBOARD_IMAGE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedClipboardImage {
    path: String,
    source_revision: String,
}

pub(super) fn persist(
    app: &tauri::AppHandle,
    bytes: Vec<u8>,
) -> Result<ManagedClipboardImage, String> {
    if bytes.is_empty() || bytes.len() > MAX_CLIPBOARD_IMAGE_BYTES {
        return Err("Clipboard image must be between 1 byte and 16 MiB".to_owned());
    }
    validation::validate_dimensions(&bytes, ImageFormat::Png)?;
    let source_revision = format!("sha256:{}", hex::encode(Sha256::digest(&bytes)));
    let target = asset_directory(app)?.join(format!("clipboard-{}.png", Uuid::new_v4()));
    fs::write(&target, bytes).map_err(|error| error.to_string())?;
    Ok(ManagedClipboardImage {
        path: target.to_string_lossy().into_owned(),
        source_revision,
    })
}
