use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::Manager;
use tauri_plugin_fs::FsExt;

pub fn asset_directory<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("spaces")
        .join("assets");
    fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    Ok(path)
}

pub(crate) fn authorize_spaces_asset<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    source: &Path,
) -> Result<PathBuf, String> {
    let assets = asset_directory(app)?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let canonical = source.canonicalize().map_err(|error| error.to_string())?;
    if canonical == assets || !canonical.starts_with(&assets) {
        return Err("Only Space-owned assets can be accessed".into());
    }
    Ok(canonical)
}

pub(crate) fn authorize_edited_image_preview<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    source: &Path,
) -> Result<PathBuf, String> {
    let previews = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("edited-image-previews")
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let canonical = source.canonicalize().map_err(|error| error.to_string())?;
    if !canonical.starts_with(&previews)
        || !canonical.is_file()
        || canonical
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("png")
    {
        return Err("Only rendered Space images can be attached".into());
    }
    Ok(canonical)
}

pub(crate) fn authorize_canvas_thumbnail<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    path: &str,
) -> Result<(), String> {
    let source = authorize_spaces_asset(app, Path::new(path))?;
    let assets = asset_directory(app)?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let name = source
        .file_name()
        .and_then(|part| part.to_str())
        .unwrap_or("");
    if !source.is_file()
        || source.parent() != Some(assets.as_path())
        || !name.starts_with("canvas-thumbnail-")
        || !name.ends_with(".jpg")
    {
        return Err("Canvas thumbnail must be a managed JPEG file".into());
    }
    Ok(())
}

pub fn authorize_destination(app: &tauri::AppHandle, destination: &Path) -> Result<(), String> {
    if !app.fs_scope().is_allowed(destination) {
        return Err("The export destination was not authorized by a user dialog".into());
    }
    Ok(())
}

pub fn authorize_source(app: &tauri::AppHandle, source: &Path) -> Result<(), String> {
    if !app.fs_scope().is_allowed(source) {
        return Err("The resource was not selected or dropped into Knovum".into());
    }
    let metadata = fs::symlink_metadata(source).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !(metadata.is_file() || metadata.is_dir()) {
        return Err("Only regular files and directories can be imported into Spaces".into());
    }
    Ok(())
}

/// Authorizes a media file the user pointed a speech feature at.
///
/// Such a file reaches the feature through exactly two provenances: an asset the Space already
/// owns, which the canvas offers directly, or a file the user handed to the application through a
/// dialog or a drop. Admitting only the second rejected every audio node on the canvas, so both
/// are stated here instead of being discovered one entry point at a time.
pub(crate) fn authorize_media_source(
    app: &tauri::AppHandle,
    source: &Path,
) -> Result<PathBuf, String> {
    let canonical = source.canonicalize().map_err(|error| error.to_string())?;
    let assets = asset_directory(app)?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let owned_by_a_space = canonical != assets && canonical.starts_with(&assets);
    // The user-granted scope holds the path the dialog or drop reported, which is not always the
    // canonical one, so each provenance is checked against the path it actually knows.
    if !owned_by_a_space && !app.fs_scope().is_allowed(source) {
        return Err(
            "Only a Space asset or a file selected or dropped into Knovum can be used here".into(),
        );
    }
    let metadata = fs::symlink_metadata(&canonical).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() == 0 {
        return Err("A media source must be a nonempty regular file".into());
    }
    Ok(canonical)
}
