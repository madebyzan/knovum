use std::path::Path;

use image::{metadata::Orientation, ImageDecoder, ImageReader};

pub(in crate::spaces::assets) fn visual_geometry(
    source: &Path,
    kind: &str,
) -> Result<Option<(u32, u32)>, String> {
    match kind {
        "image" => image_geometry(source).map(Some),
        "video" => Ok(None),
        _ => Ok(None),
    }
}

fn image_geometry(source: &Path) -> Result<(u32, u32), String> {
    let reader = ImageReader::open(source)
        .and_then(ImageReader::with_guessed_format)
        .map_err(|error| format!("failed to open image metadata: {error}"))?;
    let mut decoder = reader
        .into_decoder()
        .map_err(|error| format!("failed to read image metadata: {error}"))?;
    let orientation = decoder
        .orientation()
        .map_err(|error| format!("failed to read image orientation: {error}"))?;
    let (width, height) = decoder.dimensions();
    if matches!(
        orientation,
        Orientation::Rotate90
            | Orientation::Rotate270
            | Orientation::Rotate90FlipH
            | Orientation::Rotate270FlipH
    ) {
        Ok((height, width))
    } else {
        Ok((width, height))
    }
}
