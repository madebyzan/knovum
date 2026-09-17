use std::{
    fs,
    path::{Path, PathBuf},
};

use tauri::{AppHandle, Manager, Runtime};
use walkdir::WalkDir;

use crate::ai::{
    contracts::AgentArtifact,
    error::{AiError, AiResult},
};

mod classification;

pub fn space_workspace<R: Runtime>(app: &AppHandle<R>, space_id: &str) -> AiResult<PathBuf> {
    let space = uuid::Uuid::parse_str(space_id)
        .map_err(|_| AiError::invalid("Agent session requires a valid Space identifier"))?;
    app.path()
        .app_data_dir()
        .map(|root| {
            root.join("agent-workspaces")
                .join("spaces")
                .join(space.to_string())
        })
        .map_err(|error| AiError::internal(error.to_string()))
}

pub fn workspace<R: Runtime>(
    app: &AppHandle<R>,
    space_id: &str,
    session_id: &str,
) -> AiResult<PathBuf> {
    let session = uuid::Uuid::parse_str(session_id)
        .map_err(|_| AiError::invalid("Agent session requires a valid session identifier"))?;
    Ok(space_workspace(app, space_id)?.join(session.to_string()))
}

pub fn prepare(path: &Path) -> AiResult<()> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|error| AiError::internal(error.to_string()))?;
    }
    fs::create_dir_all(path).map_err(|error| AiError::internal(error.to_string()))
}

pub fn collect(root: &Path) -> AiResult<Vec<AgentArtifact>> {
    let mut artifacts = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|error| AiError::internal(error.to_string()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let metadata = entry
            .metadata()
            .map_err(|error| AiError::internal(error.to_string()))?;
        if hidden(entry.path(), root) {
            continue;
        }
        let path = entry.path();
        let kind = classification::classify(path)?;
        let relative_path = path
            .strip_prefix(root)
            .map_err(|error| AiError::internal(error.to_string()))?
            .to_string_lossy()
            .into_owned();
        artifacts.push(AgentArtifact {
            name: entry.file_name().to_string_lossy().into_owned(),
            relative_path,
            kind,
            size: metadata.len(),
        });
    }
    artifacts.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(artifacts)
}

pub fn discard(path: &Path) -> AiResult<()> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|error| AiError::internal(error.to_string()))?;
    }
    Ok(())
}

pub fn consume(root: &Path, relative_path: &str) -> AiResult<()> {
    let artifact = resolve(root, relative_path)?;
    fs::remove_file(artifact).map_err(|error| AiError::internal(error.to_string()))
}

pub fn resolve(root: &Path, relative_path: &str) -> AiResult<PathBuf> {
    let relative = Path::new(relative_path);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::RootDir
            )
        })
    {
        return Err(AiError::invalid("Invalid Agent artifact path"));
    }
    if !collect(root)?
        .iter()
        .any(|artifact| artifact.relative_path == relative_path)
    {
        return Err(AiError::invalid(
            "Agent artifact is not present in the session manifest",
        ));
    }
    let root = root
        .canonicalize()
        .map_err(|error| AiError::internal(error.to_string()))?;
    let candidate = root
        .join(relative)
        .canonicalize()
        .map_err(|error| AiError::invalid(error.to_string()))?;
    if !candidate.starts_with(&root) || !candidate.is_file() {
        return Err(AiError::invalid(
            "Agent artifact is outside its session workspace",
        ));
    }
    Ok(candidate)
}

fn hidden(path: &Path, root: &Path) -> bool {
    path.strip_prefix(root).is_ok_and(|relative| {
        relative
            .components()
            .any(|component| component.as_os_str().to_string_lossy().starts_with('.'))
    })
}
