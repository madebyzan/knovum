use std::path::{Path, PathBuf};

use super::paths::AppPaths;

pub(super) fn create_unique(parent: &Path, name: &str) -> Result<PathBuf, String> {
    let path = parent.join(format!(".{name}-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&path).map_err(|error| error.to_string())?;
    Ok(path)
}

pub(super) fn has_owned(paths: &AppPaths) -> bool {
    stage_scopes(paths)
        .into_iter()
        .any(
            |(root, prefix, suffix)| match find_owned(root, prefix, suffix) {
                Ok(stages) => !stages.is_empty(),
                Err(_) => true,
            },
        )
}

pub(super) fn cleanup_abandoned(paths: &AppPaths) -> Result<(), String> {
    for (root, prefix, suffix) in stage_scopes(paths) {
        for path in find_owned(root, prefix, suffix)? {
            remove_owned(&path, prefix, suffix)?;
        }
    }
    Ok(())
}

pub(super) fn cleanup_backup(path: &Path) -> Result<(), String> {
    remove_owned(path, ".backup-stage-", "")
}

pub(super) fn cleanup_restore(
    config_directory: &Path,
    data_directory: &Path,
) -> Result<(), String> {
    let mut failures = Vec::new();
    for (path, suffix) in [(config_directory, "-config"), (data_directory, "-data")] {
        if let Err(error) = remove_owned(path, ".restore-stage-", suffix) {
            failures.push(error);
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}

fn stage_scopes(paths: &AppPaths) -> [(&Path, &'static str, &'static str); 3] {
    [
        (&paths.maintenance, ".backup-stage-", ""),
        (&paths.config, ".restore-stage-", "-config"),
        (&paths.data, ".restore-stage-", "-data"),
    ]
}

fn find_owned(root: &Path, prefix: &str, suffix: &str) -> Result<Vec<PathBuf>, String> {
    let mut stages = Vec::new();
    for entry in std::fs::read_dir(root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if canonical_name(&name, prefix, suffix) {
            stages.push(entry.path());
        }
    }
    Ok(stages)
}

fn remove_owned(path: &Path, prefix: &str, suffix: &str) -> Result<(), String> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Staging path has no UTF-8 name")?;
    if !canonical_name(name, prefix, suffix) {
        return Err("Refusing to remove an unowned staging path".into());
    }
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "Owned staging path is not a real directory: {}",
            path.display()
        ));
    }
    std::fs::remove_dir_all(path).map_err(|error| error.to_string())
}

fn canonical_name(name: &str, prefix: &str, suffix: &str) -> bool {
    name.strip_prefix(prefix)
        .and_then(|value| value.strip_suffix(suffix))
        .and_then(|value| {
            uuid::Uuid::parse_str(value)
                .ok()
                .map(|parsed| (value, parsed))
        })
        .is_some_and(|(value, parsed)| parsed.to_string() == value)
}
