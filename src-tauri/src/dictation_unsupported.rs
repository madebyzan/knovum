use tauri::{plugin::TauriPlugin, Wry};

use crate::app_error::{AppError, AppResult};

pub(crate) fn plugin() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("dictation")
        .invoke_handler(tauri::generate_handler![
            get_state,
            cancel,
            dismiss_result,
            copy_result
        ])
        .build()
}

#[tauri::command]
fn get_state() -> AppResult<()> {
    Err(unsupported())
}

#[tauri::command]
fn cancel() -> AppResult<()> {
    Err(unsupported())
}

#[tauri::command]
fn dismiss_result() -> AppResult<()> {
    Err(unsupported())
}

#[tauri::command]
fn copy_result() -> AppResult<()> {
    Err(unsupported())
}

fn unsupported() -> AppError {
    AppError::new(
        "unsupported_device",
        "Native dictation is available on Apple Silicon macOS and x64 Windows",
    )
}
