use std::path::Path;

use crate::foundation::app_error::AppError;
use tauri::AppHandle;

/// Where a managed output file will be written. Reserving a name is all this is: the file appears
/// when its publisher writes it.
pub(crate) fn reserve_in(directory: &Path, prefix: &str, extension: &str) -> String {
    directory
        .join(format!("{prefix}-{}.{extension}", uuid::Uuid::new_v4()))
        .to_string_lossy()
        .into_owned()
}

pub(crate) fn reserve(app: &AppHandle, prefix: &str, extension: &str) -> Result<String, String> {
    Ok(reserve_in(
        &crate::spaces::assets::asset_directory(app)?,
        prefix,
        extension,
    ))
}

#[tauri::command]
pub fn allocate_visualization_output_path(app: AppHandle) -> Result<String, AppError> {
    Ok((reserve(&app, "generated", "png"))?)
}

#[tauri::command]
pub fn allocate_video_edit_output_path(app: AppHandle) -> Result<String, AppError> {
    Ok(reserve(&app, "edited", "mp4")?)
}
