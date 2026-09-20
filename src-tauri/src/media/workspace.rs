use std::{
    fs,
    path::{Path, PathBuf},
};

use tauri::Manager;
use uuid::Uuid;

pub(crate) struct AnalysisWorkspace {
    directory: PathBuf,
    pub wav: PathBuf,
    pub waveform: PathBuf,
    pub transcript: PathBuf,
}

impl AnalysisWorkspace {
    pub fn new(app: &tauri::AppHandle) -> Result<Self, String> {
        Self::create(&analysis_directory(app)?)
    }

    pub(super) fn create(root: &Path) -> Result<Self, String> {
        let directory = root.join(format!("media-{}", Uuid::new_v4()));
        fs::create_dir(&directory).map_err(|error| error.to_string())?;
        Ok(Self::at(directory))
    }

    pub(super) fn at(directory: PathBuf) -> Self {
        Self {
            wav: directory.join("source.wav"),
            waveform: directory.join("waveform.f32"),
            transcript: directory.join("transcript"),
            directory,
        }
    }

    pub(super) fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn cleanup(&self) -> Result<(), String> {
        match fs::remove_dir_all(&self.directory) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!("failed to remove media analysis directory: {error}")),
        }
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

pub(crate) fn analysis_directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("spaces/media-analysis");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory)
}
