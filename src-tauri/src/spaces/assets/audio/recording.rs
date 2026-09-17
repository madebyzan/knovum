use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use serde::Serialize;
use uuid::Uuid;

use super::{
    super::asset_directory,
    format::{extension_for_mime, validate_container},
};

static SESSIONS: OnceLock<Mutex<HashMap<String, RecordingSession>>> = OnceLock::new();

struct RecordingSession {
    file: File,
    temp_path: PathBuf,
    target_path: PathBuf,
    mime_type: String,
    extension: &'static str,
    bytes_written: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioRecordingSession {
    session_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedAudioRecording {
    path: String,
    extension: &'static str,
}

pub fn start(app: &tauri::AppHandle, mime_type: String) -> Result<AudioRecordingSession, String> {
    let extension = extension_for_mime(&mime_type)?;
    let session_id = Uuid::new_v4().to_string();
    let directory = asset_directory(app)?;
    let temp_path = directory.join(format!("recording-{session_id}.part"));
    let target_path = directory.join(format!("audio-{session_id}.{extension}"));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
        .map_err(|error| error.to_string())?;
    session_map()?.insert(
        session_id.clone(),
        RecordingSession {
            file,
            temp_path,
            target_path,
            mime_type,
            extension,
            bytes_written: 0,
        },
    );
    Ok(AudioRecordingSession { session_id })
}

pub fn append(session_id: &str, bytes: &[u8]) -> Result<u64, String> {
    if bytes.is_empty() {
        return session_map()?
            .get(session_id)
            .map(|session| session.bytes_written)
            .ok_or_else(|| "Audio recording session was not found".to_owned());
    }
    let mut sessions = session_map()?;
    let session = sessions
        .get_mut(session_id)
        .ok_or_else(|| "Audio recording session was not found".to_owned())?;
    session
        .file
        .write_all(bytes)
        .map_err(|error| error.to_string())?;
    session.bytes_written = session
        .bytes_written
        .checked_add(bytes.len() as u64)
        .ok_or_else(|| "Audio recording size overflow".to_owned())?;
    Ok(session.bytes_written)
}

pub fn finish(session_id: &str) -> Result<ManagedAudioRecording, String> {
    let mut session = session_map()?
        .remove(session_id)
        .ok_or_else(|| "Audio recording session was not found".to_owned())?;
    session.file.flush().map_err(|error| error.to_string())?;
    let RecordingSession {
        file,
        temp_path,
        target_path,
        mime_type,
        extension,
        bytes_written,
    } = session;
    drop(file);
    let finalized = finalize(
        &temp_path,
        &target_path,
        &mime_type,
        extension,
        bytes_written,
    );
    if finalized.is_err() {
        if let Err(cleanup_err) = fs::remove_file(&temp_path) {
            if cleanup_err.kind() != std::io::ErrorKind::NotFound {
                log::warn!(
                    "failed to remove temporary recording file {}: {cleanup_err}",
                    temp_path.display()
                );
            }
        }
    }
    finalized
}

pub fn cancel(session_id: &str) -> Result<(), String> {
    let Some(session) = session_map()?.remove(session_id) else {
        return Ok(());
    };
    drop(session.file);
    match fs::remove_file(session.temp_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

pub fn cleanup_stale(app: &tauri::AppHandle) -> Result<(), String> {
    let directory = asset_directory(app)?;
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        let stale = path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.starts_with("recording-") && name.ends_with(".part"));
        if stale {
            fs::remove_file(path).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn finalize(
    temp_path: &PathBuf,
    target_path: &PathBuf,
    mime_type: &str,
    extension: &'static str,
    bytes_written: u64,
) -> Result<ManagedAudioRecording, String> {
    if bytes_written == 0 {
        return Err("Audio recording is empty".to_owned());
    }
    validate_container(temp_path, mime_type)?;
    fs::rename(temp_path, target_path).map_err(|error| error.to_string())?;
    Ok(ManagedAudioRecording {
        path: target_path.to_string_lossy().into_owned(),
        extension,
    })
}

fn session_map() -> Result<std::sync::MutexGuard<'static, HashMap<String, RecordingSession>>, String>
{
    SESSIONS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .map_err(|_| "Audio recording session state is unavailable".to_owned())
}
