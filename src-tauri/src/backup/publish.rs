use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use super::{
    contracts::Manifest,
    database, file_publication,
    journal::{self, Journal, PublicationTarget},
    journal_validation,
    paths::AppPaths,
    settings,
};

struct Target {
    live: PathBuf,
    staged: Option<PathBuf>,
    present: bool,
}

pub async fn publish(
    paths: &AppPaths,
    manifest: &Manifest,
    config_stage: &Path,
    data_stage: &Path,
) -> Result<Option<String>, String> {
    let targets = targets(paths, manifest, config_stage, data_stage)?;
    prepare_empty_roots(&targets)?;
    for (name, _) in database::DATABASES {
        let target = targets
            .iter()
            .find(|target| target.live.ends_with(name))
            .ok_or("Restore plan is missing a database target")?;
        if target.present {
            database::validate(
                name,
                target
                    .staged
                    .as_ref()
                    .ok_or("Restore database has no staged path")?,
            )
            .await?;
        }
    }
    let store = targets
        .iter()
        .find(|target| target.live.ends_with("settings.json"))
        .ok_or("Restore plan is missing the settings target")?;
    if store.present {
        settings::validate(
            store
                .staged
                .as_ref()
                .ok_or("Restore settings have no staged path")?,
        )?;
    }
    file_publication::sync_tree(config_stage)?;
    file_publication::sync_tree(data_stage)?;
    publish(paths, targets)
}

pub fn reconcile(paths: &AppPaths) -> Result<journal::ReconcileOutcome, String> {
    journal::reconcile(paths)
}
pub fn has_unreconciled(paths: &AppPaths) -> bool {
    journal::exists(paths)
}

fn targets(
    paths: &AppPaths,
    manifest: &Manifest,
    config_directory: &Path,
    data_directory: &Path,
) -> Result<Vec<Target>, String> {
    let mut targets = Vec::new();
    for (name, key) in database::DATABASES {
        let live = paths.config.join(name);
        let present = *manifest
            .roots
            .get(*key)
            .ok_or("Restore manifest has no database root")?;
        targets.push(target(
            live.clone(),
            Some(config_directory.join(name)),
            present,
        ));
        targets.push(target(append(&live, "-wal"), None, false));
        targets.push(target(append(&live, "-shm"), None, false));
    }
    targets.push(target(
        paths.data.join("settings.json"),
        Some(data_directory.join("settings.json")),
        root(manifest, "data/store/settings.json")?,
    ));
    targets.push(target(
        paths.data.join("spaces/assets"),
        Some(data_directory.join("spaces/assets")),
        root(manifest, "data/assets/spaces")?,
    ));
    targets.push(target(
        paths.data.join("agent-extensions"),
        Some(data_directory.join("agent-extensions")),
        root(manifest, "data/assets/extensions")?,
    ));
    let secrets = paths.config.join("secrets.db");
    targets.push(target(secrets.clone(), None, false));
    targets.push(target(append(&secrets, "-wal"), None, false));
    targets.push(target(append(&secrets, "-shm"), None, false));
    Ok(targets)
}

fn root(manifest: &Manifest, name: &str) -> Result<bool, String> {
    manifest
        .roots
        .get(name)
        .copied()
        .ok_or_else(|| format!("Restore manifest has no {name} root"))
}

fn target(live: PathBuf, staged: Option<PathBuf>, present: bool) -> Target {
    Target {
        live,
        staged,
        present,
    }
}

fn append(path: &Path, suffix: &str) -> PathBuf {
    let mut value: OsString = path.as_os_str().into();
    value.push(suffix);
    value.into()
}

fn prepare_empty_roots(targets: &[Target]) -> Result<(), String> {
    for target in targets.iter().filter(|target| target.present) {
        let staged = target
            .staged
            .as_ref()
            .ok_or("Restore plan is missing staged data")?;
        if super::paths::entry_exists(staged)? {
            continue;
        }
        if target.live.ends_with("assets") || target.live.ends_with("agent-extensions") {
            std::fs::create_dir_all(staged).map_err(|error| error.to_string())?;
        } else {
            return Err(format!(
                "Backup is missing required root {}",
                target.live.display()
            ));
        }
    }
    Ok(())
}

fn publish(paths: &AppPaths, targets: Vec<Target>) -> Result<Option<String>, String> {
    let token = uuid::Uuid::new_v4();
    let entries = targets
        .into_iter()
        .map(|target| {
            Ok(PublicationTarget {
                rollback: append(&target.live, &format!(".knovum-rollback-{token}")),
                had_original: super::paths::entry_exists(&target.live)?,
                live: target.live,
                staged: target.staged,
                present: target.present,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut journal = Journal::prepared(entries);
    journal_validation::validate(paths, &journal)?;
    journal::write(paths, &journal)?;
    for entry in &journal.entries {
        if entry.had_original {
            file_publication::rename_new(&entry.live, &entry.rollback)
                .map_err(|error| rollback_error(paths, error))?;
        }
        if entry.present {
            let staged = entry
                .staged
                .as_ref()
                .ok_or_else(|| rollback_error(paths, "missing staged root"))?;
            if let Some(parent) = entry.live.parent() {
                std::fs::create_dir_all(parent).map_err(|error| rollback_error(paths, error))?;
            }
            file_publication::rename_new(staged, &entry.live)
                .map_err(|error| rollback_error(paths, error))?;
        }
    }
    if let Err(error) = journal::mark_committed(paths, &mut journal) {
        return recover_commit_error(paths, error);
    }
    match journal::reconcile(paths)? {
        journal::ReconcileOutcome::RecoveredCommit { cleanup_warning } => Ok(cleanup_warning),
        journal::ReconcileOutcome::None => {
            Err("Restore publication commit was not recorded".into())
        }
    }
}

fn recover_commit_error(
    paths: &AppPaths,
    error: impl std::fmt::Display,
) -> Result<Option<String>, String> {
    match journal::reconcile(paths) {
        Ok(journal::ReconcileOutcome::RecoveredCommit { cleanup_warning }) => {
            Ok(Some(cleanup_warning.unwrap_or_else(|| {
                format!("Restore was committed, but finalization reported an error: {error}")
            })))
        }
        Ok(journal::ReconcileOutcome::None) => Err(format!(
            "Restore publication failed and original data was restored: {error}"
        )),
        Err(rollback) => Err(format!(
            "Restore publication failed: {error}; rollback failed: {rollback}"
        )),
    }
}

fn rollback_error(paths: &AppPaths, error: impl std::fmt::Display) -> String {
    match journal::reconcile(paths) {
        Ok(journal::ReconcileOutcome::None) => {
            format!("Restore publication failed and original data was restored: {error}")
        }
        Ok(journal::ReconcileOutcome::RecoveredCommit { cleanup_warning }) => cleanup_warning
            .unwrap_or_else(|| {
                format!("Restore was committed before publication reported an error: {error}")
            }),
        Err(rollback) => {
            format!("Restore publication failed: {error}; rollback failed: {rollback}")
        }
    }
}
