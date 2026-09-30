use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::ai::error::{AiError, AiResult};

const MANIFEST_NAME: &str = ".agent-output-projection.json";
const MANIFEST_VERSION: u32 = 1;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OutputProjectionManifest {
    version: u32,
    producers: BTreeMap<String, ProducerProjection>,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProducerProjection {
    active_paths: BTreeSet<String>,
    tracked_local_paths: BTreeSet<String>,
}

impl Default for OutputProjectionManifest {
    fn default() -> Self {
        Self {
            version: MANIFEST_VERSION,
            producers: BTreeMap::new(),
        }
    }
}

pub(crate) fn record_tool(
    root: &Path,
    tool_id: &str,
    active_paths: &[PathBuf],
    local_paths: &[PathBuf],
) -> AiResult<()> {
    record(root, format!("tool:{tool_id}"), active_paths, local_paths)
}

pub(crate) fn record_message(
    root: &Path,
    occurrence_id: &str,
    active_path: &Path,
    local: bool,
) -> AiResult<()> {
    let active_paths = [active_path.to_path_buf()];
    let local_paths = local
        .then(|| active_path.to_path_buf())
        .into_iter()
        .collect::<Vec<_>>();
    record(
        root,
        format!("message:{occurrence_id}"),
        &active_paths,
        &local_paths,
    )
}

pub(super) fn retired_local_paths(root: &Path) -> AiResult<BTreeSet<String>> {
    let manifest = read(root)?;
    let active = manifest
        .producers
        .values()
        .flat_map(|producer| producer.active_paths.iter().cloned())
        .collect::<BTreeSet<_>>();
    Ok(manifest
        .producers
        .values()
        .flat_map(|producer| producer.tracked_local_paths.iter().cloned())
        .filter(|path| !active.contains(path))
        .collect())
}

pub(crate) fn obsolete_tool_artifacts(
    root: &Path,
    tool_id: &str,
    active_paths: &[PathBuf],
) -> AiResult<Vec<PathBuf>> {
    let manifest = read(root)?;
    let producer_id = format!("tool:{tool_id}");
    let Some(previous) = manifest.producers.get(&producer_id) else {
        return Ok(Vec::new());
    };
    let mut protected = relative_paths(root, active_paths)?;
    for (id, producer) in &manifest.producers {
        protected.extend(producer.tracked_local_paths.iter().cloned());
        if id != &producer_id {
            protected.extend(producer.active_paths.iter().cloned());
        }
    }
    previous
        .active_paths
        .difference(&protected)
        .map(|relative| {
            if Path::new(relative).components().count() != 1 {
                return Err(AiError::invalid(
                    "Generated Agent artifacts must be direct workspace files",
                ));
            }
            Ok(root.join(relative))
        })
        .collect()
}

fn record(
    root: &Path,
    producer_id: String,
    active_paths: &[PathBuf],
    local_paths: &[PathBuf],
) -> AiResult<()> {
    let mut manifest = read(root)?;
    let active_paths = relative_paths(root, active_paths)?;
    let local_paths = relative_paths(root, local_paths)?;
    let producer = manifest.producers.entry(producer_id).or_default();
    producer.active_paths = active_paths;
    producer.tracked_local_paths.extend(local_paths);
    write(root, &manifest)
}

fn relative_paths(root: &Path, paths: &[PathBuf]) -> AiResult<BTreeSet<String>> {
    let root = root
        .canonicalize()
        .map_err(|error| AiError::internal(error.to_string()))?;
    paths
        .iter()
        .map(|path| {
            let path = path
                .canonicalize()
                .map_err(|error| AiError::invalid(error.to_string()))?;
            let relative = path
                .strip_prefix(&root)
                .map_err(|_| AiError::invalid("Agent output is outside its session workspace"))?;
            validate_path(relative)?;
            relative
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| AiError::invalid("Agent output path must use valid UTF-8"))
        })
        .collect()
}

fn read(root: &Path) -> AiResult<OutputProjectionManifest> {
    let path = root.join(MANIFEST_NAME);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Default::default()),
        Err(error) => return Err(AiError::internal(error.to_string())),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(AiError::invalid(
            "Agent output projection manifest must be a regular workspace file",
        ));
    }
    let bytes = fs::read(&path).map_err(|error| AiError::internal(error.to_string()))?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
        AiError::invalid(format!("Agent output projection is invalid: {error}"))
    })?;
    let version = value
        .get("version")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| AiError::invalid("Agent output projection version is required"))?;
    if version != u64::from(MANIFEST_VERSION) {
        return Err(AiError::invalid(format!(
            "Unsupported Agent output projection version; this build accepts v{MANIFEST_VERSION}"
        )));
    }
    let manifest: OutputProjectionManifest = serde_json::from_value(value).map_err(|error| {
        AiError::invalid(format!("Agent output projection is invalid: {error}"))
    })?;
    for producer in manifest.producers.values() {
        for relative in producer
            .active_paths
            .iter()
            .chain(&producer.tracked_local_paths)
        {
            validate_relative(relative)?;
        }
    }
    Ok(manifest)
}

fn write(root: &Path, manifest: &OutputProjectionManifest) -> AiResult<()> {
    let bytes =
        serde_json::to_vec(manifest).map_err(|error| AiError::internal(error.to_string()))?;
    let pending = root.join(format!(
        ".agent-output-projection-{}.partial",
        uuid::Uuid::new_v4()
    ));
    let target = root.join(MANIFEST_NAME);
    let result = (|| {
        let mut file = fs::File::create(&pending)?;
        use std::io::Write;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&pending, target)
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&pending);
        return Err(AiError::internal(format!(
            "Cannot save Agent output projection: {error}"
        )));
    }
    Ok(())
}

fn validate_relative(relative: &str) -> AiResult<&Path> {
    let path = Path::new(relative);
    validate_path(path)?;
    Ok(path)
}

fn validate_path(path: &Path) -> AiResult<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(AiError::invalid("Invalid Agent output projection path"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_is_optional_strict_and_rejects_symlinks() {
        let root = std::env::temp_dir().join(format!("agent-projection-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        assert!(retired_local_paths(&root).unwrap().is_empty());

        fs::write(root.join(MANIFEST_NAME), r#"{"version":2,"producers":{}}"#).unwrap();
        assert!(retired_local_paths(&root)
            .unwrap_err()
            .to_string()
            .contains("accepts v1"));

        fs::write(
            root.join(MANIFEST_NAME),
            r#"{"version":1,"producers":{},"extra":true}"#,
        )
        .unwrap();
        assert!(retired_local_paths(&root).is_err());

        #[cfg(unix)]
        {
            fs::remove_file(root.join(MANIFEST_NAME)).unwrap();
            let outside = root.with_extension("json");
            fs::write(&outside, r#"{"version":1,"producers":{}}"#).unwrap();
            std::os::unix::fs::symlink(&outside, root.join(MANIFEST_NAME)).unwrap();
            assert!(retired_local_paths(&root).is_err());
            fs::remove_file(outside).unwrap();
        }
        fs::remove_dir_all(root).unwrap();
    }
}
