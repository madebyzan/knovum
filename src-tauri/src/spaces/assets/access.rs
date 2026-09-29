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
