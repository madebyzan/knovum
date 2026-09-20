use super::super::PreparedAsset;
use super::recording::{append, cancel, finish, start, AudioRecordingSession};

#[tauri::command]
pub async fn start_audio_recording(
    app: tauri::AppHandle,
    mime_type: String,
    display_name: String,
) -> Result<AudioRecordingSession, String> {
    crate::blocking::run(move || start(&app, mime_type, display_name)).await
}

#[tauri::command]
pub async fn append_audio_chunk(session_id: String, bytes: Vec<u8>) -> Result<u64, String> {
    crate::blocking::run(move || append(&session_id, &bytes)).await
}

#[tauri::command]
pub async fn finalize_audio_recording(session_id: String) -> Result<PreparedAsset, String> {
    crate::blocking::run(move || finish(&session_id)).await
}

#[tauri::command]
pub async fn cancel_audio_recording(session_id: String) -> Result<(), String> {
    crate::blocking::run(move || cancel(&session_id)).await
}
