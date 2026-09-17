use std::{fs, path::PathBuf};

use image::ImageFormat;
use uuid::Uuid;

use super::{super::access::authorize_source, validation};
use crate::spaces::assets::asset_directory;

const MAX_EDITOR_IMAGE_BYTES: usize = 20 * 1024 * 1024;

pub(super) fn import_file(app: &tauri::AppHandle, source: PathBuf) -> Result<String, String> {
    authorize_source(app, &source)?;
    if !source.is_file() {
        return Err("Editor images must be regular files".into());
    }
    let size = fs::metadata(&source)
        .map_err(|error| error.to_string())?
        .len();
    let size = usize::try_from(size).map_err(|_| "Editor image size exceeds this platform")?;
    validate_size(size)?;
    let bytes = fs::read(source).map_err(|error| error.to_string())?;
    let media_type = validation::detect_media_type(&bytes)?;
    let format = format_for_media_type(media_type)?;
    persist(app, bytes, format)
}

pub(super) fn persist_bytes(
    app: &tauri::AppHandle,
    bytes: Vec<u8>,
    declared_media_type: &str,
) -> Result<String, String> {
    validate_size(bytes.len())?;
    let format = format_for_media_type(declared_media_type)?;
    let detected_media_type = validation::detect_media_type(&bytes)?;
    if detected_media_type != declared_media_type {
        return Err("Editor image bytes do not match the declared type".to_owned());
    }
    persist(app, bytes, format)
}

fn persist(app: &tauri::AppHandle, bytes: Vec<u8>, format: ImageFormat) -> Result<String, String> {
    validation::validate_dimensions(&bytes, format)?;
    let extension = extension_for_format(format);
    let target = asset_directory(app)?.join(format!("editor-{}.{extension}", Uuid::new_v4()));
    fs::write(&target, bytes).map_err(|error| error.to_string())?;
    Ok(target.to_string_lossy().into_owned())
}

pub(super) fn validate_size(size: usize) -> Result<(), String> {
    if size == 0 || size > MAX_EDITOR_IMAGE_BYTES {
        Err("Editor images must be between 1 byte and 20 MiB".to_owned())
    } else {
        Ok(())
    }
}

pub(super) fn format_for_media_type(media_type: &str) -> Result<ImageFormat, String> {
    match media_type {
        "image/png" => Ok(ImageFormat::Png),
        "image/jpeg" => Ok(ImageFormat::Jpeg),
        "image/gif" => Ok(ImageFormat::Gif),
        "image/webp" => Ok(ImageFormat::WebP),
        _ => Err("Unsupported editor image type".to_owned()),
    }
}

pub(super) const fn extension_for_format(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        ImageFormat::Gif => "gif",
        ImageFormat::WebP => "webp",
        _ => unreachable!(),
    }
}
