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
    super::{asset_directory, snapshot_to_imported, PreparedAsset},
    format::{extension_for_mime, validate_container},
};

static SESSIONS: OnceLock<Mutex<HashMap<String, RecordingSession>>> = OnceLock::new();

struct RecordingSession {
    file: File,
    staged: StagedRecording,
    target_path: PathBuf,
    mime_type: String,
    name: String,
    bytes_written: u64,
}

// The staging file has exactly one owner, including early flush/inspection failures.
struct StagedRecording(PathBuf);

impl StagedRecording {
    fn remove(&self) -> Result<(), String> {
        match fs::remove_file(&self.0) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }
}

impl Drop for StagedRecording {
    fn drop(&mut self) {
        if let Err(error) = self.remove() {
            log::warn!(
                "failed to remove temporary recording {}: {error}",
                self.0.display()
            );
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioRecordingSession {
    session_id: String,
}

pub fn start(
    app: &tauri::AppHandle,
    mime_type: String,
    display_name: String,
) -> Result<AudioRecordingSession, String> {
    let extension = extension_for_mime(&mime_type)?;
    let name = display_name.trim();
    if name.is_empty() || name.len() > 480 || name.contains(['/', '\\', '\0', '\n', '\r']) {
        return Err("Audio recording display name is invalid".into());
    }
    let session_id = Uuid::new_v4().to_string();
    let directory = asset_directory(app)?;
    let temp_path = directory.join(format!("recording-{session_id}.part"));
    let target_path = directory.join(format!("audio-{session_id}.{extension}"));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
        .map_err(|error| error.to_string())?;
    let session = RecordingSession {
        file,
        staged: StagedRecording(temp_path),
        target_path,
        mime_type,
        name: format!("{name}.{extension}"),
        bytes_written: 0,
    };
    session_map()?.insert(session_id.clone(), session);
    Ok(AudioRecordingSession { session_id })
}

pub fn append(session_id: &str, bytes: &[u8]) -> Result<u64, String> {
    let mut sessions = session_map()?;
    let session = sessions
        .get_mut(session_id)
        .ok_or_else(|| "Audio recording session was not found".to_owned())?;
    let next_size = session
        .bytes_written
        .checked_add(bytes.len() as u64)
        .ok_or_else(|| "Audio recording size overflow".to_owned())?;
    session
        .file
        .write_all(bytes)
        .map_err(|error| error.to_string())?;
    session.bytes_written = next_size;
    Ok(next_size)
}

pub fn finish(session_id: &str) -> Result<PreparedAsset, String> {
    let session = session_map()?
        .remove(session_id)
        .ok_or_else(|| "Audio recording session was not found".to_owned())?;
    let RecordingSession {
        file,
        staged,
        target_path,
        mime_type,
        name,
        bytes_written,
    } = session;
    close_recording_file(file)?;
    if bytes_written == 0 {
        return Err("Audio recording is empty".into());
    }
    validate_container(&staged.0, &mime_type)?;
    let mut snapshot = crate::file_inspection::inspect_file(&staged.0, &name)?;
    if snapshot.kind != "audio" || snapshot.size != bytes_written {
        return Err("Recorded asset does not satisfy the audio publication contract".into());
    }
    let revision = crate::file_inspection::content_revision(&staged.0)?;
    snapshot.path = target_path.to_string_lossy().into_owned();
    let asset = snapshot_to_imported(snapshot, revision)?;
    fs::rename(&staged.0, target_path).map_err(|error| error.to_string())?;
    Ok(asset)
}

fn close_recording_file(mut file: File) -> Result<(), String> {
    file.flush().map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())
}

pub fn cancel(session_id: &str) -> Result<(), String> {
    let Some(session) = session_map()?.remove(session_id) else {
        return Ok(());
    };
    drop(session.file);
    session.staged.remove()
}

pub fn cleanup_stale(app: &tauri::AppHandle) -> Result<(), String> {
    for entry in fs::read_dir(asset_directory(app)?).map_err(|error| error.to_string())? {
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

fn session_map() -> Result<std::sync::MutexGuard<'static, HashMap<String, RecordingSession>>, String>
{
    SESSIONS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .map_err(|_| "Audio recording session state is unavailable".to_owned())
}
