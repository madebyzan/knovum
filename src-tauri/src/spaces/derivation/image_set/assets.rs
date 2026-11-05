use std::{fs, io::Cursor, path::PathBuf};

use sha2::{Digest, Sha256};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::ImageSet;

/// Decoder limits for reading back a member. The bytes are produced in process, so these bound the
/// memory one decode may claim rather than police an untrusted input.
const MAX_IMAGE_DIMENSION: u32 = 16_384;
const MAX_IMAGE_PIXELS: u64 = 25_000_000;
const MAX_DECODE_BYTES: u64 = 160 * 1024 * 1024;

pub struct PreparedMember {
    pub path: PathBuf,
    pub digest: String,
    pub size: usize,
    pub width: u32,
    pub height: u32,
}

/// Writes every member PNG into the asset directory, removing what it wrote if any member fails.
pub fn prepare(
    app: &AppHandle,
    set: &ImageSet,
    members: Vec<Vec<u8>>,
    cancellation: &CancellationToken,
) -> Result<Vec<PreparedMember>, String> {
    let directory = crate::spaces::assets::asset_directory(app)?;
    let mut prepared = Vec::with_capacity(members.len());
    for bytes in members {
        match prepare_one(&directory, set.member, bytes, cancellation) {
            Ok(member) => prepared.push(member),
            Err(error) => {
                remove(&prepared);
                return Err(error);
            }
        }
    }
    Ok(prepared)
}

/// Writes one member and reads back the dimensions its node publishes.
fn prepare_one(
    directory: &std::path::Path,
    member: &str,
    bytes: Vec<u8>,
    cancellation: &CancellationToken,
) -> Result<PreparedMember, String> {
    if cancellation.is_cancelled() {
        return Err("Image set derivation was cancelled".into());
    }
    let decoded = decode_png(&bytes)?;
    if cancellation.is_cancelled() {
        return Err("Image set derivation was cancelled".into());
    }
    let path = directory.join(format!("{member}-{}.png", Uuid::new_v4()));
    if let Err(error) = fs::write(&path, &bytes) {
        let _ = fs::remove_file(&path);
        return Err(error.to_string());
    }
    Ok(PreparedMember {
        path,
        digest: format!("sha256:{}", hex::encode(Sha256::digest(&bytes))),
        size: bytes.len(),
        width: decoded.width(),
        height: decoded.height(),
    })
}

fn decode_png(bytes: &[u8]) -> Result<image::DynamicImage, String> {
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    limits.max_alloc = Some(MAX_DECODE_BYTES);
    reader.limits(limits);
    let decoded = reader
        .decode()
        .map_err(|error| format!("Generated image is not a valid PNG: {error}"))?;
    if u64::from(decoded.width()) * u64::from(decoded.height()) > MAX_IMAGE_PIXELS {
        return Err("Generated image dimensions exceed the image set limit".into());
    }
    Ok(decoded)
}

pub fn remove(members: &[PreparedMember]) {
    for member in members {
        let _ = fs::remove_file(&member.path);
    }
}
