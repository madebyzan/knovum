use std::{
    fs,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

use crate::ai::contracts::AgentArtifact;
use crate::foundation::app_error::{AppError, AppResult};

mod classification;
mod output_projection;

pub(crate) use output_projection::{obsolete_tool_artifacts, record_message, record_tool};

pub fn prepare(path: &Path) -> AppResult<()> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|error| AppError::internal(error.to_string()))?;
    }
    fs::create_dir_all(path).map_err(|error| AppError::internal(error.to_string()))
}

pub fn collect(root: &Path) -> AppResult<Vec<AgentArtifact>> {
    let retired_local_paths = output_projection::retired_local_paths(root)?;
    let mut artifacts = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|error| AppError::internal(error.to_string()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        if hidden(entry.path(), root) {
            continue;
        }
        let path = entry.path();
        let relative_path = path
            .strip_prefix(root)
            .map_err(|error| AppError::internal(error.to_string()))?
            .to_string_lossy()
            .into_owned();
        if retired_local_paths.contains(&relative_path) {
            continue;
        }
        let metadata = entry
            .metadata()
            .map_err(|error| AppError::internal(error.to_string()))?;
        let kind = classification::classify(path)?;
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

pub fn discard(path: &Path) -> AppResult<()> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|error| AppError::internal(error.to_string()))?;
    }
    Ok(())
}

pub fn consume(root: &Path, relative_path: &str) -> AppResult<()> {
    let artifact = resolve(root, relative_path)?;
    fs::remove_file(artifact).map_err(|error| AppError::internal(error.to_string()))
}

pub fn resolve(root: &Path, relative_path: &str) -> AppResult<PathBuf> {
    let manifest = collect(root)?;
    let artifact = manifest
        .iter()
        .find(|artifact| artifact.relative_path == relative_path)
        .ok_or_else(|| {
            AppError::invalid("Agent artifact is not present in the session manifest")
        })?;
    resolve_collected(root, artifact)
}

pub fn resolve_collected(root: &Path, artifact: &AgentArtifact) -> AppResult<PathBuf> {
    let relative_path = &artifact.relative_path;
    let relative = Path::new(relative_path);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::RootDir
            )
        })
    {
        return Err(AppError::invalid("Invalid Agent artifact path"));
    }
    let root = root
        .canonicalize()
        .map_err(|error| AppError::internal(error.to_string()))?;
    let candidate = root
        .join(relative)
        .canonicalize()
        .map_err(|error| AppError::invalid(error.to_string()))?;
    if !candidate.starts_with(&root) || !candidate.is_file() {
        return Err(AppError::invalid(
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
