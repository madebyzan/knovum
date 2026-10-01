use super::model::{
    DocumentResource, MAX_RESOURCE_BYTES, MAX_RESOURCE_DIMENSION, MAX_RESOURCE_PIXELS,
};
use crate::spaces::assets::{asset_directory, authorize_spaces_asset};
use sha2::{Digest, Sha256};
use std::{fs, io::Cursor, path::Path};

pub(super) enum ResourceError {
    Unsupported(String),
    Storage(String),
}

/// A lossless PNG raster is the one visual-input contract. The original remains in the document.
pub(super) fn save(
    app: &tauri::AppHandle,
    source: &Path,
    bytes: &[u8],
    claim_managed_path: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<DocumentResource, ResourceError> {
    let (bytes, width, height) = normalize(bytes).map_err(ResourceError::Unsupported)?;
    store(app, source, &bytes, width, height, claim_managed_path).map_err(ResourceError::Storage)
}

fn normalize(bytes: &[u8]) -> Result<(Vec<u8>, u32, u32), String> {
    if bytes.len() as u64 > MAX_RESOURCE_BYTES {
        return Err("Document image exceeds 32 MiB".into());
    }
    let format = image::guess_format(bytes).map_err(|error| error.to_string())?;
    match format {
        image::ImageFormat::Png
        | image::ImageFormat::Jpeg
        | image::ImageFormat::WebP
        | image::ImageFormat::Tiff
        | image::ImageFormat::Bmp => {}
        _ => return Err("Document image format is not supported".into()),
    }
    let (width, height) = image::ImageReader::with_format(Cursor::new(bytes), format)
        .into_dimensions()
        .map_err(|error| error.to_string())?;
    if u64::from(width) * u64::from(height) > MAX_RESOURCE_PIXELS {
        return Err("Document image exceeds 24 megapixels".into());
    }
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_RESOURCE_DIMENSION);
    limits.max_image_height = Some(MAX_RESOURCE_DIMENSION);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|error| format!("Document image decode failed: {error}"))?;
    let mut output = Cursor::new(Vec::new());
    image
        .write_to(&mut output, image::ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    let bytes = output.into_inner();
    if bytes.len() as u64 > MAX_RESOURCE_BYTES {
        return Err("Normalized document image exceeds 32 MiB".into());
    }
    Ok((bytes, width, height))
}

fn store(
    app: &tauri::AppHandle,
    source: &Path,
    bytes: &[u8],
    width: u32,
    height: u32,
    claim_managed_path: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<DocumentResource, String> {
    let id = format!("{:x}", Sha256::digest(bytes));
    let path = resource_path(&asset_directory(app)?, source, &id);
    let resource = store_at(&path, id, bytes, width, height, claim_managed_path)?;
    authorize_spaces_asset(app, &path)?;
    Ok(resource)
}

fn resource_path(directory: &Path, source: &Path, digest: &str) -> std::path::PathBuf {
    let owner = format!(
        "{:x}",
        Sha256::digest(source.as_os_str().as_encoded_bytes())
    );
    directory.join(format!("document-{owner}-{digest}.png"))
}

fn store_at(
    path: &Path,
    id: String,
    bytes: &[u8],
    width: u32,
    height: u32,
    claim_managed_path: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<DocumentResource, String> {
    claim_managed_path(path)?;
    if path.exists() {
        let existing = read_bounded(path)?;
        if format!("{:x}", Sha256::digest(&existing)) != id {
            return Err("Document resource digest does not match its stored bytes".into());
        }
    } else {
        let partial = path.with_extension(format!("{}.partial", uuid::Uuid::new_v4()));
        claim_managed_path(&partial)?;
        let result = fs::write(&partial, bytes)
            .and_then(|()| fs::rename(&partial, path))
            .map_err(|error| error.to_string());
        if let Err(error) = result {
            return if partial.exists() {
                match fs::remove_file(&partial) {
                    Ok(()) => Err(error),
                    Err(cleanup) => Err(format!("{error}; partial cleanup failed: {cleanup}")),
                }
            } else {
                Err(error)
            };
        }
    }
    Ok(DocumentResource {
        id,
        path: path.to_string_lossy().into_owned(),
        media_type: "image/png".into(),
        width,
        height,
        byte_size: bytes.len() as u64,
    })
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.take(MAX_RESOURCE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_RESOURCE_BYTES {
        return Err("Stored document image exceeds 32 MiB".into());
    }
    Ok(bytes)
}

pub(super) fn verified_path(
    app: &tauri::AppHandle,
    resource: &DocumentResource,
) -> Result<std::path::PathBuf, String> {
    let path = authorize_spaces_asset(app, Path::new(&resource.path))?;
    let bytes = read_bounded(&path)?;
    if bytes.len() as u64 != resource.byte_size
        || format!("{:x}", Sha256::digest(&bytes)) != resource.id
    {
        return Err("Document resource bytes do not match the published manifest".into());
    }
    Ok(path)
}

pub(super) fn save_render(
    app: &tauri::AppHandle,
    source: &Path,
    input: &crate::documents::pdf_input::PdfInput,
    claim_managed_path: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<DocumentResource, String> {
    let raster = crate::documents::rendering::render(source, input)?;
    // The native adapter already owns bounded PNG generation; do not decode and encode it again.
    if raster.bytes.len() as u64 > MAX_RESOURCE_BYTES {
        return Err("Rendered PDF input exceeds 32 MiB".into());
    }
    store(
        app,
        source,
        &raster.bytes,
        raster.width,
        raster.height,
        claim_managed_path,
    )
}

#[cfg(test)]
mod tests;
