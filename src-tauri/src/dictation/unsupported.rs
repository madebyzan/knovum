use tauri::{plugin::TauriPlugin, Wry};

use crate::foundation::app_error::{AppError, AppResult};

pub(crate) fn plugin() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("dictation")
        .invoke_handler(tauri::generate_handler![
            get_state,
            cancel,
            dismiss_transcription,
            copy_transcription,
            get_dictation_shortcut,
            set_dictation_shortcut,
            reset_dictation_shortcut
        ])
        .build()
}

#[tauri::command]
fn get_dictation_shortcut() -> AppResult<super::contracts::DictationShortcutStatus> {
    Err(unsupported())
}

#[tauri::command]
fn set_dictation_shortcut(
    _shortcut: String,
) -> AppResult<super::contracts::DictationShortcutStatus> {
    Err(unsupported())
}

#[tauri::command]
fn reset_dictation_shortcut() -> AppResult<super::contracts::DictationShortcutStatus> {
    Err(unsupported())
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
fn dismiss_transcription() -> AppResult<()> {
    Err(unsupported())
}

#[tauri::command]
fn copy_transcription() -> AppResult<()> {
    Err(unsupported())
}

fn unsupported() -> AppError {
    AppError::new(
        "unsupported_device",
        "Desktop dictation is available on Apple Silicon macOS and x64 Windows",
    )
}
