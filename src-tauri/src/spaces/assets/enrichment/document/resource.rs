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
    bytes: &[u8],
) -> Result<DocumentResource, ResourceError> {
    let (bytes, width, height) = normalize(bytes).map_err(ResourceError::Unsupported)?;
    store(app, &bytes, width, height).map_err(ResourceError::Storage)
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
    bytes: &[u8],
    width: u32,
    height: u32,
) -> Result<DocumentResource, String> {
    let id = format!("{:x}", Sha256::digest(bytes));
    let path = asset_directory(app)?.join(format!("document-{id}.png"));
    if path.exists() {
        let canonical = authorize_spaces_asset(app, &path)?;
        let existing = read_bounded(&canonical)?;
        if format!("{:x}", Sha256::digest(&existing)) != id {
            return Err("Document resource digest does not match its stored bytes".into());
        }
    } else {
        let partial = path.with_extension(format!("{}.partial", uuid::Uuid::new_v4()));
        fs::write(&partial, bytes).map_err(|error| error.to_string())?;
        fs::rename(&partial, &path).map_err(|error| error.to_string())?;
    }
    authorize_spaces_asset(app, &path)?;
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
) -> Result<DocumentResource, String> {
    let raster = crate::documents::rendering::render(source, input)?;
    // The native adapter already owns bounded PNG generation; do not decode and encode it again.
    if raster.bytes.len() as u64 > MAX_RESOURCE_BYTES {
        return Err("Rendered PDF input exceeds 32 MiB".into());
    }
    store(app, &raster.bytes, raster.width, raster.height)
}
