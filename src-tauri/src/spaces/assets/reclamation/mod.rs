use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, Runtime};

mod reference_graph;

pub(crate) use reference_graph::{
    for_nodes as referenced_paths_for_nodes, for_space as referenced_paths_for_space,
};

pub(crate) async fn reclaim_orphan_managed_assets<R: Runtime>(
    app: &AppHandle<R>,
    pool: &SqlitePool,
) -> Result<(), String> {
    let referenced = reference_graph::all(pool).await?;
    let root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("spaces")
        .join("assets");
    if !root.exists() {
        return Ok(());
    }
    let mut candidates = Vec::new();
    for entry in fs::read_dir(&root).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if !tree_is_referenced(&path, &referenced) {
            candidates.push(path.to_string_lossy().into_owned());
        }
    }
    remove_managed_paths(app, candidates)
}

fn tree_is_referenced(path: &Path, referenced: &BTreeSet<String>) -> bool {
    referenced
        .iter()
        .any(|candidate| Path::new(candidate).starts_with(path))
}

pub(crate) async fn unreferenced_paths(
    pool: &SqlitePool,
    candidates: BTreeSet<String>,
) -> Result<Vec<String>, String> {
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    let referenced = reference_graph::all(pool).await?;
    Ok(candidates
        .into_iter()
        .filter(|path| !referenced.contains(path))
        .collect())
}

fn remove_managed_paths<R: Runtime>(app: &AppHandle<R>, paths: Vec<String>) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }
    let root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("spaces")
        .join("assets");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    for path in paths {
        let candidate = PathBuf::from(path);
        if !candidate.exists() {
            continue;
        }
        let metadata = fs::symlink_metadata(&candidate).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() || !(metadata.is_file() || metadata.is_dir()) {
            return Err("Managed asset is not a regular file or directory".to_owned());
        }
        let canonical = candidate
            .canonicalize()
            .map_err(|error| error.to_string())?;
        if !canonical.starts_with(&root) || canonical == root {
            return Err("Managed asset escaped the Space asset directory".to_owned());
        }
        if canonical.is_dir() {
            fs::remove_dir_all(canonical).map_err(|error| error.to_string())?;
        } else {
            fs::remove_file(canonical).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
