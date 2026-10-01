use crate::app_error::AppError;
use std::path::PathBuf;

use super::{clipboard, editor};

#[tauri::command]
pub async fn persist_editor_image(
    app: tauri::AppHandle,
    bytes: Vec<u8>,
    mime_type: String,
) -> Result<String, AppError> {
    Ok((crate::blocking::run(move || editor::persist_bytes(&app, bytes, &mime_type)).await)?)
}

#[tauri::command]
pub async fn persist_clipboard_image(
    app: tauri::AppHandle,
    bytes: Vec<u8>,
) -> Result<clipboard::ManagedClipboardImage, AppError> {
    Ok((crate::blocking::run(move || clipboard::persist(&app, bytes)).await)?)
}

#[tauri::command]
pub async fn import_editor_image(
    app: tauri::AppHandle,
    source_path: String,
) -> Result<String, AppError> {
    Ok(
        (crate::blocking::run(move || editor::import_file(&app, PathBuf::from(source_path)))
            .await)?,
    )
}
