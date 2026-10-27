use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::foundation::app_error::{AppError, AppResult};

const MANIFEST_NAME: &str = ".agent-output-projection.json";
const MANIFEST_VERSION: u32 = 2;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OutputProjectionManifest {
    version: u32,
    producers: BTreeMap<String, ProducerProjection>,
}

/// The workspace files one producer — a tool call or a message occurrence — currently stands
/// behind. A tool that reports new content retires the files it reported before.
#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProducerProjection {
    active_paths: BTreeSet<String>,
}

impl Default for OutputProjectionManifest {
    fn default() -> Self {
        Self {
            version: MANIFEST_VERSION,
            producers: BTreeMap::new(),
        }
    }
}

pub(crate) fn record_tool(root: &Path, tool_id: &str, active_paths: &[PathBuf]) -> AppResult<()> {
    record(root, format!("tool:{tool_id}"), active_paths)
}

pub(crate) fn record_message(
    root: &Path,
    occurrence_id: &str,
    active_path: &Path,
) -> AppResult<()> {
    record(
        root,
        format!("message:{occurrence_id}"),
        &[active_path.to_path_buf()],
    )
}

pub(crate) fn obsolete_tool_artifacts(
    root: &Path,
    tool_id: &str,
    active_paths: &[PathBuf],
) -> AppResult<Vec<PathBuf>> {
    let manifest = read(root)?;
    let producer_id = format!("tool:{tool_id}");
    let Some(previous) = manifest.producers.get(&producer_id) else {
        return Ok(Vec::new());
    };
    let mut protected = relative_paths(root, active_paths)?;
    for (id, producer) in &manifest.producers {
        if id != &producer_id {
            protected.extend(producer.active_paths.iter().cloned());
        }
    }
    previous
        .active_paths
        .difference(&protected)
        .map(|relative| {
            if Path::new(relative).components().count() != 1 {
                return Err(AppError::invalid(
                    "Generated Agent artifacts must be direct workspace files",
                ));
            }
            Ok(root.join(relative))
        })
        .collect()
}

fn record(root: &Path, producer_id: String, active_paths: &[PathBuf]) -> AppResult<()> {
    let mut manifest = read(root)?;
    let active_paths = relative_paths(root, active_paths)?;
    manifest
        .producers
        .entry(producer_id)
        .or_default()
        .active_paths = active_paths;
    write(root, &manifest)
}

fn relative_paths(root: &Path, paths: &[PathBuf]) -> AppResult<BTreeSet<String>> {
    let root = root
        .canonicalize()
        .map_err(|error| AppError::internal(error.to_string()))?;
    paths
        .iter()
        .map(|path| {
            let path = path
                .canonicalize()
                .map_err(|error| AppError::invalid(error.to_string()))?;
            let relative = path
                .strip_prefix(&root)
                .map_err(|_| AppError::invalid("Agent output is outside its session workspace"))?;
            validate_path(relative)?;
            relative
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| AppError::invalid("Agent output path must use valid UTF-8"))
        })
        .collect()
}

fn read(root: &Path) -> AppResult<OutputProjectionManifest> {
    let path = root.join(MANIFEST_NAME);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Default::default()),
        Err(error) => return Err(AppError::internal(error.to_string())),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(AppError::invalid(
            "Agent output projection manifest must be a regular workspace file",
        ));
    }
    let bytes = fs::read(&path).map_err(|error| AppError::internal(error.to_string()))?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
        AppError::invalid(format!("Agent output projection is invalid: {error}"))
    })?;
    let version = value
        .get("version")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| AppError::invalid("Agent output projection version is required"))?;
    if version != u64::from(MANIFEST_VERSION) {
        return Err(AppError::invalid(format!(
            "Unsupported Agent output projection version; this build accepts v{MANIFEST_VERSION}"
        )));
    }
    let manifest: OutputProjectionManifest = serde_json::from_value(value).map_err(|error| {
        AppError::invalid(format!("Agent output projection is invalid: {error}"))
    })?;
    for producer in manifest.producers.values() {
        for relative in &producer.active_paths {
            validate_relative(relative)?;
        }
    }
    Ok(manifest)
}

fn write(root: &Path, manifest: &OutputProjectionManifest) -> AppResult<()> {
    let bytes =
        serde_json::to_vec(manifest).map_err(|error| AppError::internal(error.to_string()))?;
    let pending = root.join(format!(
        ".agent-output-projection-{}.partial",
        uuid::Uuid::new_v4()
    ));
    let target = root.join(MANIFEST_NAME);
    let projection = (|| {
        let mut file = fs::File::create(&pending)?;
        use std::io::Write;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&pending, target)
    })();
    if let Err(error) = projection {
        let _ = fs::remove_file(&pending);
        return Err(AppError::internal(format!(
            "Cannot save Agent output projection: {error}"
        )));
    }
    Ok(())
}

fn validate_relative(relative: &str) -> AppResult<&Path> {
    let path = Path::new(relative);
    validate_path(path)?;
    Ok(path)
}

fn validate_path(path: &Path) -> AppResult<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(AppError::invalid("Invalid Agent output projection path"));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
