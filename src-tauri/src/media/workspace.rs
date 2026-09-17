use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use tauri::Manager;

pub(crate) struct AnalysisWorkspace {
    directory: PathBuf,
    pub wav: PathBuf,
    pub waveform: PathBuf,
    pub transcript: PathBuf,
}

impl AnalysisWorkspace {
    pub fn new(app: &tauri::AppHandle) -> Result<Self, String> {
        let root = analysis_directory(app)?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        let directory = root.join(format!("media-{}-{nonce}", std::process::id()));
        fs::create_dir(&directory).map_err(|error| error.to_string())?;
        Ok(Self {
            wav: directory.join("source.wav"),
            waveform: directory.join("waveform.f32"),
            transcript: directory.join("transcript"),
            directory,
        })
    }

    pub fn cleanup(&self) -> Result<(), String> {
        if self.directory.exists() {
            fs::remove_dir_all(&self.directory)
                .map_err(|error| format!("failed to remove media analysis directory: {error}"))?;
        }
        Ok(())
    }
}

pub fn cleanup_stale(app: &tauri::AppHandle) -> Result<(), String> {
    let directory = analysis_directory(app)?;
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|error| error.to_string())?;
        } else {
            fs::remove_file(path).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn analysis_directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("spaces/media-analysis");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory)
}
