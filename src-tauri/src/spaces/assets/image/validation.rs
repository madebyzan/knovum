use image::{GenericImageView, ImageFormat};

const MAX_IMAGE_PIXELS: u64 = 25_000_000;

pub(super) fn detect_media_type(bytes: &[u8]) -> Result<&'static str, String> {
    infer::get(bytes)
        .map(|kind| kind.mime_type())
        .ok_or_else(|| "Image type could not be verified".to_owned())
}

pub(super) fn validate_dimensions(bytes: &[u8], format: ImageFormat) -> Result<(), String> {
    let image =
        image::load_from_memory_with_format(bytes, format).map_err(|error| error.to_string())?;
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > MAX_IMAGE_PIXELS {
        return Err("Space image dimensions are invalid or too large".to_owned());
    }
    Ok(())
}
