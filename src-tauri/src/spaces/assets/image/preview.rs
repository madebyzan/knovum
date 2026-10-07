use std::{
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use tauri::Manager;

use super::document::ImageDocument;

pub(in crate::spaces::assets) fn prepare(
    app: &tauri::AppHandle,
    document: ImageDocument,
) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("edited-image-previews");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let key = format!(
        "{}:{}:{}",
        document.source_path, document.source_revision, document.edit
    );
    let directory = directory.join(hex::encode(Sha256::digest(key.as_bytes())));
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let stem = Path::new(&document.file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or("Image file name is invalid")?;
    let display_name: String = stem
        .chars()
        .filter(|value| value.is_alphanumeric() || matches!(value, ' ' | '-' | '_'))
        .take(80)
        .collect();
    let display_name = if display_name.is_empty() {
        "Image"
    } else {
        &display_name
    };
    let path = directory.join(format!("{display_name}.png"));
    let source = super::super::authorize_spaces_asset(app, Path::new(&document.source_path))?;
    crate::file_inspection::verify_content_revision(&source, &document.source_revision)?;
    if path.is_file() {
        return Ok(path);
    }
    let rendered = crate::images::render_source_png(&source, &document.edit)?;
    crate::file_inspection::verify_content_revision(&source, &document.source_revision)?;
    let temporary = directory.join(format!("{}.partial", uuid::Uuid::new_v4()));
    fs::write(&temporary, rendered.png).map_err(|error| error.to_string())?;
    fs::rename(&temporary, &path).map_err(|error| error.to_string())?;
    Ok(path)
}
