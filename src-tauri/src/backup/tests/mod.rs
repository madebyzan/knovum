use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use super::{
    database,
    journal::{self, Journal, JournalEntry, JournalPhase, ReconcileOutcome},
    paths::AppPaths,
    settings, staging,
};

struct Fixture {
    root: PathBuf,
    paths: AppPaths,
}

impl Fixture {
    fn new() -> Self {
        let root =
            std::env::temp_dir().join(format!("knovum-backup-test-{}", uuid::Uuid::new_v4()));
        let config = root.join("config");
        let data = root.join("data");
        let maintenance = config.join("maintenance");
        fs::create_dir_all(&maintenance).expect("maintenance directory");
        fs::create_dir_all(&data).expect("data directory");
        Self {
            root,
            paths: AppPaths {
                config,
                data,
                maintenance,
            },
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn prepared_journal_restores_original_data() {
    let fixture = Fixture::new();
    let token = uuid::Uuid::new_v4();
    let live = fixture.paths.data.join("settings.json");
    let rollback = rollback(&live, token);
    fs::write(&live, b"replacement").expect("replacement data");
    fs::write(&rollback, b"original").expect("rollback data");
    let journal = journal_with(&fixture.paths, token, JournalPhase::Prepared, &live);
    journal::write(&fixture.paths, &journal).expect("prepared journal");

    assert!(matches!(
        journal::reconcile(&fixture.paths),
        Ok(ReconcileOutcome::None)
    ));
    assert_eq!(fs::read(&live).expect("restored data"), b"original");
    assert!(!rollback.exists());
}

#[test]
fn committed_cleanup_is_idempotent_and_keeps_published_data() {
    let fixture = Fixture::new();
    let token = uuid::Uuid::new_v4();
    let live = fixture.paths.data.join("settings.json");
    let rollback = rollback(&live, token);
    fs::write(&live, b"published").expect("published data");
    fs::write(&rollback, b"original").expect("rollback data");
    let journal = journal_with(&fixture.paths, token, JournalPhase::Committed, &live);
    journal::write(&fixture.paths, &journal).expect("committed journal");

    assert!(matches!(
        journal::reconcile(&fixture.paths),
        Ok(ReconcileOutcome::RecoveredCommit {
            cleanup_warning: None
        })
    ));
    assert_eq!(fs::read(&live).expect("published data"), b"published");
    assert!(matches!(
        journal::reconcile(&fixture.paths),
        Ok(ReconcileOutcome::None)
    ));
}

#[test]
fn invalid_journal_target_is_rejected_without_touching_live_data() {
    let fixture = Fixture::new();
    let outside = fixture.root.join("outside.txt");
    fs::write(&outside, b"keep").expect("outside data");
    let journal = Journal {
        phase: JournalPhase::Prepared,
        entries: vec![JournalEntry {
            live: outside.clone(),
            staged: None,
            rollback: rollback(&outside, uuid::Uuid::new_v4()),
            had_original: true,
            present: false,
        }],
    };
    journal::write(&fixture.paths, &journal).expect("invalid journal persisted");

    assert!(journal::reconcile(&fixture.paths).is_err());
    assert_eq!(fs::read(outside).expect("outside data preserved"), b"keep");
}

#[test]
fn settings_validation_rejects_duplicate_presets_and_unknown_preferences() {
    let fixture = Fixture::new();
    let path = fixture.paths.data.join("settings.json");
    fs::write(&path, duplicate_presets()).expect("settings file");
    assert!(settings::validate(&path).is_err());
    fs::write(&path, preferences_with_unknown_field()).expect("settings file");
    assert!(settings::validate(&path).is_err());
}

#[test]
fn active_space_package_job_blocks_backup_admission() {
    let lease =
        crate::spaces::package::control::begin_test_job("00000000-0000-4000-8000-000000000001")
            .expect("package job");
    assert!(crate::spaces::backup::reserve_maintenance().is_err());
    drop(lease);
    let admission = crate::spaces::backup::reserve_maintenance().expect("idle engines reserved");
    assert!(crate::spaces::package::control::begin_test_job(
        "00000000-0000-4000-8000-000000000002"
    )
    .is_err());
    drop(admission);
    let _lease =
        crate::spaces::package::control::begin_test_job("00000000-0000-4000-8000-000000000002")
            .expect("failed scheduling reopens package admission");
}

#[test]
fn interrupted_owned_stages_are_reclaimed() {
    let fixture = Fixture::new();
    let token = uuid::Uuid::new_v4();
    let stages = [
        fixture
            .paths
            .maintenance
            .join(format!(".backup-stage-{token}")),
        fixture
            .paths
            .config
            .join(format!(".restore-stage-{token}-config")),
        fixture
            .paths
            .data
            .join(format!(".restore-stage-{token}-data")),
    ];
    for stage in &stages {
        fs::create_dir(stage).expect("owned stage");
        fs::write(stage.join("partial"), b"data").expect("staged data");
    }
    assert!(staging::has_owned(&fixture.paths));
    staging::cleanup_abandoned(&fixture.paths).expect("cleanup owned stages");
    assert!(stages.iter().all(|stage| !stage.exists()));
}

#[test]
fn unowned_stage_like_directory_is_preserved() {
    let fixture = Fixture::new();
    let unowned = fixture.paths.maintenance.join(".backup-stage-not-a-uuid");
    fs::create_dir(&unowned).expect("unowned directory");
    staging::cleanup_abandoned(&fixture.paths).expect("ignore unowned directory");
    assert!(unowned.is_dir());
}

#[cfg(unix)]
#[test]
fn owned_stage_symlink_is_rejected_without_touching_target() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new();
    let target = fixture.root.join("symlink-target");
    fs::create_dir(&target).expect("symlink target");
    fs::write(target.join("keep"), b"data").expect("target data");
    let stage = fixture
        .paths
        .maintenance
        .join(format!(".backup-stage-{}", uuid::Uuid::new_v4()));
    symlink(&target, &stage).expect("owned stage symlink");

    assert!(staging::cleanup_abandoned(&fixture.paths).is_err());
    assert_eq!(
        fs::read(target.join("keep")).expect("target preserved"),
        b"data"
    );
}

fn journal_with(
    paths: &AppPaths,
    token: uuid::Uuid,
    phase: JournalPhase,
    changed: &Path,
) -> Journal {
    let mut live = Vec::new();
    for (name, _) in database::DATABASES {
        let database = paths.config.join(name);
        live.extend([
            database.clone(),
            append(&database, "-wal"),
            append(&database, "-shm"),
        ]);
    }
    let secrets = paths.config.join("secrets.db");
    live.extend([
        secrets.clone(),
        append(&secrets, "-wal"),
        append(&secrets, "-shm"),
    ]);
    live.extend([
        paths.data.join("settings.json"),
        paths.data.join("spaces/assets"),
        paths.data.join("agent-extensions"),
    ]);
    Journal {
        phase,
        entries: live
            .into_iter()
            .map(|path| JournalEntry {
                had_original: path == changed,
                rollback: rollback(&path, token),
                live: path,
                staged: None,
                present: false,
            })
            .collect(),
    }
}

fn rollback(path: &Path, token: uuid::Uuid) -> PathBuf {
    append(path, &format!(".knovum-rollback-{token}"))
}

fn append(path: &Path, suffix: &str) -> PathBuf {
    let mut value: OsString = path.as_os_str().into();
    value.push(suffix);
    value.into()
}

fn duplicate_presets() -> &'static [u8] {
    br#"{"creation.library":{"schemaVersion":1,"value":{"generationPresets":[{"id":"same","name":"One","actionId":"write","target":{"type":"agent","runtimeKind":"codex"},"parameters":{}},{"id":"same","name":"Two","actionId":"write","target":{"type":"agent","runtimeKind":"codex"},"parameters":{}}]}}}"#
}

fn preferences_with_unknown_field() -> &'static [u8] {
    br#"{"preferences":{"schemaVersion":1,"value":{"theme":"system","density":"compact","textSize":"default","locale":"system","backgroundTaskNotifications":true,"workbench":{"sidebarVisible":true,"sidebarWidth":224,"rightSidebarVisible":false,"rightSidebarWidth":380},"unknown":true}}}"#
}
