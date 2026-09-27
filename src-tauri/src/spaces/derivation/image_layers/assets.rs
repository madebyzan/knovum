use std::{fs, io::Cursor, path::PathBuf};

use sha2::{Digest, Sha256};
use tauri::AppHandle;
use uuid::Uuid;

const MAX_SOURCE_BYTES: usize = 10 * 1024 * 1024;
const MAX_LAYER_BYTES: usize = 20 * 1024 * 1024;
const MAX_IMAGE_DIMENSION: u32 = 16_384;
const MAX_IMAGE_PIXELS: u64 = 25_000_000;

pub struct PreparedLayer {
    pub path: PathBuf,
    pub digest: String,
    pub size: usize,
    pub width: u32,
    pub height: u32,
}

pub fn source_data_uri(bytes: &[u8]) -> Result<String, String> {
    use base64::{engine::general_purpose::STANDARD, Engine};

    if bytes.is_empty() || bytes.len() > MAX_SOURCE_BYTES {
        return Err("Image layer source must be a PNG no larger than 10 MB".into());
    }
    decode_png(bytes, "Image layer source")?;
    Ok(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}

pub fn prepare(app: &AppHandle, layers: Vec<Vec<u8>>) -> Result<Vec<PreparedLayer>, String> {
    let directory = crate::spaces::assets::asset_directory(app)?;
    let mut prepared = Vec::with_capacity(layers.len());
    for bytes in layers {
        let prepared_layer = prepare_one(&directory, bytes);
        match prepared_layer {
            Ok(layer) => prepared.push(layer),
            Err(error) => {
                remove(&prepared);
                return Err(error);
            }
        }
    }
    Ok(prepared)
}

fn prepare_one(directory: &std::path::Path, bytes: Vec<u8>) -> Result<PreparedLayer, String> {
    if bytes.is_empty() || bytes.len() > MAX_LAYER_BYTES {
        return Err("fal returned an invalid layer size".into());
    }
    let decoded = decode_png(&bytes, "fal layer")?;
    if !decoded.color().has_alpha() {
        return Err("fal returned a layer without an alpha channel".into());
    }
    let path = directory.join(format!("layer-{}.png", Uuid::new_v4()));
    if let Err(error) = fs::write(&path, &bytes) {
        let _ = fs::remove_file(&path);
        return Err(error.to_string());
    }
    Ok(PreparedLayer {
        path,
        digest: format!("sha256:{:x}", Sha256::digest(&bytes)),
        size: bytes.len(),
        width: decoded.width(),
        height: decoded.height(),
    })
}

fn decode_png(bytes: &[u8], label: &str) -> Result<image::DynamicImage, String> {
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    limits.max_alloc = Some(160 * 1024 * 1024);
    reader.limits(limits);
    let decoded = reader
        .decode()
        .map_err(|error| format!("{label} is not a valid PNG: {error}"))?;
    if decoded.width() == 0
        || decoded.height() == 0
        || u64::from(decoded.width()) * u64::from(decoded.height()) > MAX_IMAGE_PIXELS
    {
        return Err(format!("{label} dimensions exceed the image layer limit"));
    }
    Ok(decoded)
}

pub fn remove(layers: &[PreparedLayer]) {
    for layer in layers {
        let _ = fs::remove_file(&layer.path);
    }
}
