use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_fs::FsExt;
use tokio::sync::watch;

use crate::foundation::app_error::{AppError, AppResult};

mod admission;
mod archive;
mod archive_format;
pub(crate) mod contracts;
mod database;
mod file_publication;
mod files;
mod journal;
mod journal_validation;
mod maintenance;
mod paths;
mod publish;
mod settings;
mod staging;

#[cfg(test)]
mod tests;

use contracts::{MaintenanceReceipt, PendingOperation};
use paths::AppPaths;

struct MaintenanceState {
    started: AtomicBool,
    running: AtomicBool,
    outcome: watch::Sender<Option<Result<(), String>>>,
}

impl Default for MaintenanceState {
    fn default() -> Self {
        let (outcome, _) = watch::channel(None);
        Self {
            started: AtomicBool::new(false),
            running: AtomicBool::new(false),
            outcome,
        }
    }
}

pub fn plugin<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("backup")
        .invoke_handler(tauri::generate_handler![
            schedule_backup,
            schedule_restore,
            get_maintenance_receipt,
            acknowledge_maintenance_receipt,
            await_data_maintenance,
        ])
        .setup(|app, _| {
            app.manage(MaintenanceState::default());
            Ok(())
        })
        .on_event(|app, event| {
            if matches!(event, tauri::RunEvent::Ready) {
                start(app);
            }
        })
        .build()
}

pub(crate) async fn ready<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let mut outcome = app.state::<MaintenanceState>().outcome.subscribe();
    loop {
        if let Some(completion) = outcome.borrow_and_update().clone() {
            return completion;
        }
        outcome
            .changed()
            .await
            .map_err(|_| "Data maintenance state closed unexpectedly".to_owned())?;
    }
}

pub(crate) fn is_running<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.state::<MaintenanceState>()
        .running
        .load(Ordering::Acquire)
}

fn start<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<MaintenanceState>();
    if state.started.swap(true, Ordering::AcqRel) {
        return;
    }
    let has_work = maintenance::is_required(app);
    state.running.store(has_work, Ordering::Release);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if has_work {
            if let Err(error) = crate::desktop::set_maintenance_progress(&app, Some(5)) {
                log::warn!("failed to set maintenance progress: {error}");
            }
        }
        let maintenance = maintenance::run(&app).await;
        let succeeded = matches!(&maintenance, Ok(true));
        if let Err(error) = &maintenance {
            maintenance::show_error(&app, error);
        }
        let outcome = maintenance.map(|_| ());
        let state = app.state::<MaintenanceState>();
        state.outcome.send_replace(Some(outcome));
        state.running.store(false, Ordering::Release);
        if has_work {
            if let Err(error) = crate::desktop::finish_maintenance_progress(&app, succeeded) {
                log::warn!("failed to finish maintenance progress: {error}");
            }
        }
    });
}

#[tauri::command]
async fn await_data_maintenance<R: Runtime>(app: AppHandle<R>) -> AppResult<()> {
    ready(&app).await.map_err(AppError::from)
}

#[tauri::command]
async fn schedule_backup<R: Runtime>(app: AppHandle<R>, destination: String) -> AppResult<()> {
    let path = authorized_path(&app, &destination, false)?;
    if paths::entry_exists(&path)? {
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(AppError::invalid(
                "Backup destination must be a regular file",
            ));
        }
    }
    let _admission = admission::reserve(&app).await?;
    schedule(
        &app,
        PendingOperation::Backup {
            destination: path_string(path)?,
        },
    )
    .map_err(AppError::from)
}

#[tauri::command]
async fn schedule_restore<R: Runtime>(app: AppHandle<R>, source: String) -> AppResult<()> {
    let path = authorized_path(&app, &source, true)?;
    let metadata = std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(AppError::invalid("Backup source must be a regular file"));
    }
    let _admission = admission::reserve(&app).await?;
    schedule(
        &app,
        PendingOperation::Restore {
            source: path_string(path)?,
        },
    )
    .map_err(AppError::from)
}

fn schedule<R: Runtime>(app: &AppHandle<R>, operation: PendingOperation) -> Result<(), String> {
    let paths = AppPaths::resolve(app)?;
    if paths::entry_exists(&paths.pending())? {
        return Err("A data maintenance operation is already pending".into());
    }
    if paths::entry_exists(&paths.receipt())? {
        return Err(
            "Dismiss the previous data maintenance receipt before starting another operation"
                .into(),
        );
    }
    paths::write_pending(&paths, &operation)?;
    crate::desktop::restart_for_maintenance(app)
}

#[tauri::command]
fn get_maintenance_receipt<R: Runtime>(app: AppHandle<R>) -> AppResult<Option<MaintenanceReceipt>> {
    paths::read_receipt(&AppPaths::resolve(&app)?).map_err(AppError::from)
}

#[tauri::command]
fn acknowledge_maintenance_receipt<R: Runtime>(app: AppHandle<R>) -> AppResult<()> {
    paths::acknowledge_receipt(&AppPaths::resolve(&app)?).map_err(AppError::from)
}

fn authorized_path<R: Runtime>(
    app: &AppHandle<R>,
    value: &str,
    must_exist: bool,
) -> Result<PathBuf, String> {
    let raw = Path::new(value);
    if !app.fs_scope().is_allowed(raw) {
        return Err("Selected path is not authorized".into());
    }
    if must_exist {
        let metadata = std::fs::symlink_metadata(raw).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err("Selected source cannot be a symbolic link".into());
        }
        raw.canonicalize().map_err(|error| error.to_string())
    } else {
        let parent = raw
            .parent()
            .ok_or("Backup destination has no parent directory")?;
        let parent = parent.canonicalize().map_err(|error| error.to_string())?;
        Ok(parent.join(
            raw.file_name()
                .ok_or("Backup destination has no filename")?,
        ))
    }
}

fn path_string(path: PathBuf) -> Result<String, String> {
    path.into_os_string()
        .into_string()
        .map_err(|_| "Selected path is not valid UTF-8".into())
}
