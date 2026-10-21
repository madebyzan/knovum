use std::path::Path;

use serde_json::Value;

use super::{files, ManagedSource};
use crate::file_inspection::verify_content_revision;
use crate::images::Encoding;

/// Writes the picture as it is currently seen: the asset with its pending edit
/// applied, in the encoding the destination asked for. The revision is checked
/// on both sides of the render, so an asset replaced while encoding cannot
/// leave a mixed result behind.
pub(super) fn write(
    app: &tauri::AppHandle,
    source: &ManagedSource,
    edit: &Value,
    encoding: Encoding,
    destination: &Path,
) -> Result<(), String> {
    verify_content_revision(&source.path, &source.revision)?;
    let bytes = crate::images::render_source_as(&source.path, edit, encoding)?;
    verify_content_revision(&source.path, &source.revision)?;
    files::write(app, destination, &bytes)
}
