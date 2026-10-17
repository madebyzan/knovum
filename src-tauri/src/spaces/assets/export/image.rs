use std::path::Path;

use serde_json::Value;

use super::{files, ManagedSource};
use crate::file_inspection::verify_content_revision;

/// The encodings a destination can ask a picture for. PNG keeps every pixel;
/// JPEG is the small, lossy one. Lossless WebP is neither, so it is absent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Encoding {
    Png,
    Jpeg,
}

impl Encoding {
    pub(super) const fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
        }
    }
}

/// Writes the picture as it is currently seen: the asset with its pending edit
/// applied. The revision is checked on both sides of the render, so an asset
/// replaced while encoding cannot leave a mixed result behind.
pub(super) fn write(
    app: &tauri::AppHandle,
    source: &ManagedSource,
    payload: &Value,
    encoding: Encoding,
    destination: &Path,
) -> Result<(), String> {
    verify_content_revision(&source.path, &source.revision)?;
    let edit = payload.get("imageEdit").cloned().unwrap_or(Value::Null);
    let bytes = match encoding {
        Encoding::Png => crate::images::render_source_png(&source.path, &edit)?.png,
        Encoding::Jpeg => crate::images::render_source_jpeg(&source.path, &edit)?,
    };
    verify_content_revision(&source.path, &source.revision)?;
    files::write(app, destination, &bytes)
}
