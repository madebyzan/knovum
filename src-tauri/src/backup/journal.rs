use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    io::Write,
    path::{Path, PathBuf},
};

use super::{
    file_publication, journal_validation,
    paths::{self, AppPaths},
};

const FILE: &str = "restore-publication-v1.json";
const JOURNAL_LIMIT: u64 = 64 * 1024;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Journal {
    pub phase: JournalPhase,
    pub entries: Vec<JournalEntry>,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum JournalPhase {
    Prepared,
    Committed,
}

pub enum ReconcileOutcome {
    None,
    RecoveredCommit { cleanup_warning: Option<String> },
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JournalEntry {
    pub live: PathBuf,
    pub staged: Option<PathBuf>,
    pub rollback: PathBuf,
    pub had_original: bool,
    pub present: bool,
}

impl Journal {
    pub fn prepared(entries: Vec<JournalEntry>) -> Self {
        Self {
            phase: JournalPhase::Prepared,
            entries,
        }
    }
}

pub fn write(paths: &AppPaths, journal: &Journal) -> Result<(), String> {
    let path = path(paths);
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let bytes = serde_json::to_vec(journal).map_err(|error| error.to_string())?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    file.write_all(&bytes).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    file_publication::replace_file(&temporary, &path)
}

pub fn mark_committed(paths: &AppPaths, journal: &mut Journal) -> Result<(), String> {
    journal.phase = JournalPhase::Committed;
    write(paths, journal)
}

pub fn reconcile(paths: &AppPaths) -> Result<ReconcileOutcome, String> {
    let path = path(paths);
    if !paths::entry_exists(&path)? {
        return Ok(ReconcileOutcome::None);
    }
    let journal: Journal = serde_json::from_slice(&paths::read_bounded(&path, JOURNAL_LIMIT)?)
        .map_err(|error| format!("Restore publication journal is invalid: {error}"))?;
    journal_validation::validate(paths, &journal)?;
    if matches!(journal.phase, JournalPhase::Prepared) {
        rollback(paths, &journal)?;
        file_publication::remove(&path)?;
        return Ok(ReconcileOutcome::None);
    }
    let cleanup_warning = cleanup(paths, &journal).err().or_else(|| {
        file_publication::remove(&path).err()
    }).map(|error| format!("Restored data is active, but obsolete recovery files could not be removed: {error}"));
    Ok(ReconcileOutcome::RecoveredCommit { cleanup_warning })
}

pub fn exists(paths: &AppPaths) -> bool {
    paths::entry_exists(&path(paths)).unwrap_or(true)
}

fn rollback(paths: &AppPaths, journal: &Journal) -> Result<(), String> {
    let mut failures = Vec::new();
    for entry in journal.entries.iter().rev() {
        if present(&entry.rollback, "inspect rollback target", &mut failures) {
            if present(&entry.live, "inspect live target", &mut failures) {
                record(
                    file_publication::remove(&entry.live),
                    "remove partially published target",
                    &mut failures,
                );
            }
            record(
                file_publication::rename_new(&entry.rollback, &entry.live),
                "restore original target",
                &mut failures,
            );
        } else if !entry.had_original && present(&entry.live, "inspect new target", &mut failures) {
            record(
                file_publication::remove(&entry.live),
                "remove new target",
                &mut failures,
            );
        }
        if let Some(staged) = &entry.staged {
            if present(staged, "inspect staged target", &mut failures) {
                record(
                    file_publication::remove(staged),
                    "remove staged target",
                    &mut failures,
                );
            }
        }
    }
    remove_stage_roots(paths, journal, &mut failures);
    finish(failures)
}

fn cleanup(paths: &AppPaths, journal: &Journal) -> Result<(), String> {
    let mut failures = Vec::new();
    for entry in &journal.entries {
        if present(&entry.rollback, "inspect rollback target", &mut failures) {
            record(
                file_publication::remove(&entry.rollback),
                "remove rollback target",
                &mut failures,
            );
        }
        if let Some(staged) = &entry.staged {
            if present(staged, "inspect staged target", &mut failures) {
                record(
                    file_publication::remove(staged),
                    "remove staged target",
                    &mut failures,
                );
            }
        }
    }
    remove_stage_roots(paths, journal, &mut failures);
    finish(failures)
}

fn remove_stage_roots(paths: &AppPaths, journal: &Journal, failures: &mut Vec<String>) {
    let mut roots = HashSet::new();
    for staged in journal
        .entries
        .iter()
        .filter_map(|entry| entry.staged.as_ref())
    {
        for root in [&paths.config, &paths.data] {
            if let Ok(relative) = staged.strip_prefix(root) {
                if let Some(std::path::Component::Normal(component)) = relative.components().next()
                {
                    if component.to_string_lossy().starts_with(".restore-stage-") {
                        roots.insert(root.join(component));
                    }
                }
            }
        }
    }
    for root in roots {
        if present(&root, "inspect staging root", failures) {
            record(
                file_publication::remove(&root),
                "remove staging root",
                failures,
            );
        }
    }
}

fn record(outcome: Result<(), String>, operation: &str, failures: &mut Vec<String>) {
    if let Err(error) = outcome {
        failures.push(format!("{operation}: {error}"));
    }
}
fn present(path: &Path, operation: &str, failures: &mut Vec<String>) -> bool {
    match paths::entry_exists(path) {
        Ok(present) => present,
        Err(error) => {
            failures.push(format!("{operation}: {error}"));
            false
        }
    }
}
fn finish(failures: Vec<String>) -> Result<(), String> {
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}
fn path(paths: &AppPaths) -> PathBuf {
    paths.maintenance.join(FILE)
}
