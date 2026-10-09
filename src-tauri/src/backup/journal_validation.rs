use std::{
    collections::HashSet,
    ffi::OsString,
    path::{Component, Path, PathBuf},
};

use super::{journal::Journal, paths::AppPaths};

pub fn validate(paths: &AppPaths, journal: &Journal) -> Result<(), String> {
    let allowed = allowed_targets(paths);
    if journal.entries.len() != allowed.len() {
        return Err("Restore journal target count is invalid".into());
    }
    let mut found = HashSet::new();
    let mut stage_token = None;
    let mut rollback_token = None;
    for entry in &journal.entries {
        if !allowed.contains(&entry.live) || !found.insert(entry.live.clone()) {
            return Err("Restore journal contains an invalid target".into());
        }
        validate_live_path(paths, &entry.live)?;
        validate_rollback(&entry.live, &entry.rollback, &mut rollback_token)?;
        let name = entry
            .live
            .file_name()
            .ok_or("Restore target has no filename")?
            .to_string_lossy();
        let excluded = name == "secrets.db"
            || name.starts_with("secrets.db-")
            || name.ends_with("-wal")
            || name.ends_with("-shm");
        if excluded && (entry.present || entry.staged.is_some()) {
            return Err("Restore journal may not publish excluded database state".into());
        }
        if entry.present && entry.staged.is_none() {
            return Err("Restore journal is missing staged data".into());
        }
        if let Some(staged) = &entry.staged {
            validate_staged(paths, &entry.live, staged, &mut stage_token)?;
        }
    }
    Ok(())
}

fn allowed_targets(paths: &AppPaths) -> HashSet<PathBuf> {
    let mut allowed = HashSet::new();
    for name in ["spaces.db", "extensions.db", "ai-usage.db", "secrets.db"] {
        let database = paths.config.join(name);
        allowed.extend([
            database.clone(),
            append(&database, "-wal"),
            append(&database, "-shm"),
        ]);
    }
    allowed.extend([
        paths.data.join("settings.json"),
        paths.data.join("spaces/assets"),
        paths.data.join("agent-extensions"),
    ]);
    allowed
}

fn validate_rollback(
    live: &Path,
    rollback: &Path,
    token: &mut Option<String>,
) -> Result<(), String> {
    let live_name = live
        .file_name()
        .ok_or("Restore target has no filename")?
        .to_string_lossy();
    let rollback_name = rollback
        .file_name()
        .ok_or("Restore rollback target has no filename")?
        .to_string_lossy();
    let expected = format!("{live_name}.knovum-rollback-");
    let Some(current) = rollback_name.strip_prefix(&expected) else {
        return Err("Restore journal contains an invalid rollback path".into());
    };
    if rollback.parent() != live.parent()
        || uuid::Uuid::parse_str(current).is_err()
        || token.as_ref().is_some_and(|value| value != current)
    {
        return Err("Restore journal contains an invalid rollback path".into());
    }
    reject_symlink(rollback)?;
    token.get_or_insert_with(|| current.to_owned());
    Ok(())
}

fn validate_staged(
    paths: &AppPaths,
    live: &Path,
    staged: &Path,
    token: &mut Option<String>,
) -> Result<(), String> {
    let (root, suffix, scope) = staging_destination(paths, live)?;
    let relative = staged
        .strip_prefix(root)
        .map_err(|_| "Restore journal staging path is outside application data")?;
    let mut components = relative.components();
    let Some(Component::Normal(stage)) = components.next() else {
        return Err("Restore journal staging path is invalid".into());
    };
    let stage_text = stage
        .to_str()
        .ok_or("Restore journal staging path is not UTF-8")?;
    let stage_uuid = stage_text
        .strip_prefix(".restore-stage-")
        .and_then(|value| value.strip_suffix(scope))
        .ok_or("Restore journal staging path is invalid")?;
    uuid::Uuid::parse_str(stage_uuid)
        .map_err(|_| "Restore journal staging transaction is invalid")?;
    if token.as_ref().is_some_and(|current| current != stage_uuid) {
        return Err("Restore journal mixes staging transactions".into());
    }
    token.get_or_insert_with(|| stage_uuid.to_owned());
    let remainder: PathBuf = components.collect();
    if remainder != suffix {
        return Err("Restore journal staging target does not match its live target".into());
    }
    reject_staging_symlinks(root, staged)?;
    Ok(())
}

fn staging_destination<'a>(
    paths: &'a AppPaths,
    live: &Path,
) -> Result<(&'a Path, PathBuf, &'static str), String> {
    for (target, suffix) in [
        (
            paths.data.join("settings.json"),
            PathBuf::from("settings.json"),
        ),
        (
            paths.data.join("spaces/assets"),
            PathBuf::from("spaces/assets"),
        ),
        (
            paths.data.join("agent-extensions"),
            PathBuf::from("agent-extensions"),
        ),
    ] {
        if live == target {
            return Ok((&paths.data, suffix, "-data"));
        }
    }
    let filename = live.file_name().ok_or("Restore target has no filename")?;
    if live.parent() == Some(paths.config.as_path()) {
        return Ok((&paths.config, PathBuf::from(filename), "-config"));
    }
    Err("Restore journal staging target is invalid".into())
}

fn reject_staging_symlinks(root: &Path, staged: &Path) -> Result<(), String> {
    for path in staged.ancestors().take_while(|path| *path != root) {
        reject_symlink(path)?;
    }
    Ok(())
}

fn validate_live_path(paths: &AppPaths, live: &Path) -> Result<(), String> {
    let data_targets = [
        paths.data.join("settings.json"),
        paths.data.join("spaces/assets"),
        paths.data.join("agent-extensions"),
    ];
    let root = if data_targets.iter().any(|target| target == live) {
        &paths.data
    } else {
        &paths.config
    };
    for path in live.ancestors().take_while(|path| *path != root) {
        reject_symlink(path)?;
    }
    Ok(())
}

fn reject_symlink(path: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err("Restore journal references a symbolic link".into())
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn append(path: &Path, suffix: &str) -> PathBuf {
    let mut value: OsString = path.as_os_str().into();
    value.push(suffix);
    value.into()
}
