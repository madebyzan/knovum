use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::Manager;

pub fn preview_directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("spaces")
        .join("previews");
    fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    Ok(path)
}

pub fn preview_path(app: &tauri::AppHandle, asset: &Path, suffix: &str) -> Result<PathBuf, String> {
    let name = asset
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or("Asset has no usable file name")?;
    Ok(preview_directory(app)?.join(format!("{}-{name}.{suffix}", asset_identity(asset))))
}

pub fn remove_generated_previews(app: &tauri::AppHandle, asset: &Path) -> Result<(), String> {
    let directory = preview_directory(app)?;
    let prefix = format!("{}-", asset_identity(asset));
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry.file_name().to_string_lossy().starts_with(&prefix) {
            fs::remove_file(entry.path()).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn asset_identity(asset: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(asset.to_string_lossy().as_bytes());
    let digest = hex::encode(hasher.finalize());
    digest[..16].to_owned()
}
