mod observers;
mod presentation;

use crate::{app_error::AppError, desktop_task::DesktopTask};
use std::collections::{BTreeMap, HashMap};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use tauri::{AppHandle, Manager, Runtime};

#[derive(Default)]
pub(crate) struct TaskHub {
    groups: Mutex<HashMap<&'static str, BTreeMap<String, DesktopTask>>>,
    notifications: AtomicBool,
    chinese: AtomicBool,
}

pub(crate) fn install(app: &AppHandle) {
    app.manage(TaskHub::default());
    observers::install(app);
}

pub(crate) fn set_locale<R: Runtime>(app: &AppHandle<R>, locale: &str) {
    app.state::<TaskHub>()
        .chinese
        .store(locale == "zh-CN", Ordering::SeqCst);
}

#[tauri::command]
pub(crate) fn set_task_notifications<R: Runtime>(app: AppHandle<R>, enabled: bool) {
    app.state::<TaskHub>()
        .notifications
        .store(enabled, Ordering::SeqCst);
}

#[tauri::command]
pub(crate) fn project_session_tasks<R: Runtime>(
    app: AppHandle<R>,
    tasks: Vec<DesktopTask>,
) -> Result<(), AppError> {
    if tasks.len() > 1000 {
        return Err("too many session tasks".into());
    }
    for task in &tasks {
        if task.id.is_empty()
            || task.id.len() > 128
            || task.progress.is_some_and(|value| value > 100)
            || task.started_at < 0
        {
            return Err("invalid session task projection".into());
        }
    }
    publish(&app, "session", tasks).map_err(Into::into)
}

pub(crate) fn publish<R: Runtime>(
    app: &AppHandle<R>,
    owner: &'static str,
    tasks: Vec<DesktopTask>,
) -> Result<(), String> {
    let hub = app.state::<TaskHub>();
    let mut groups = hub
        .groups
        .lock()
        .map_err(|_| "desktop task state is unavailable")?;
    let next: BTreeMap<_, _> = tasks
        .into_iter()
        .map(|task| (task.id.clone(), task))
        .collect();
    if let Some(previous) = groups.get(owner) {
        for (id, task) in &next {
            let rust_owned = owner == "session"
                && groups
                    .iter()
                    .any(|(group, tasks)| *group != "session" && tasks.contains_key(id));
            if !rust_owned
                && previous
                    .get(id)
                    .is_some_and(|before| before.state.active() && !task.state.active())
            {
                presentation::notify(app, &hub, task);
            }
        }
    }
    groups.insert(owner, next);
    // Rust engine projections override session projections with the same producer identity.
    let mut merged = groups.get("session").cloned().unwrap_or_default();
    for (_, tasks) in groups.iter().filter(|(owner, _)| **owner != "session") {
        merged.extend(tasks.clone());
    }
    presentation::progress(
        app,
        merged.values().filter(|task| {
            matches!(
                task.state,
                crate::desktop_task::DesktopTaskState::Queued
                    | crate::desktop_task::DesktopTaskState::Running
            )
        }),
    )
}

pub(crate) fn maintenance<R: Runtime>(
    app: &AppHandle<R>,
    progress: Option<u8>,
    outcome: Option<bool>,
) -> Result<(), String> {
    let hub = app.state::<TaskHub>();
    let previous_start = hub
        .groups
        .lock()
        .map_err(|_| "desktop task state is unavailable")?
        .get("maintenance")
        .and_then(|tasks| tasks.get("data-maintenance"))
        .map(|task| task.started_at);
    let started_at = match previous_start {
        Some(value) => value,
        None => crate::clock::unix_time_millis()?,
    };
    let state = match outcome {
        None => crate::desktop_task::DesktopTaskState::Running,
        Some(true) => crate::desktop_task::DesktopTaskState::Succeeded,
        Some(false) => crate::desktop_task::DesktopTaskState::Failed,
    };
    publish(
        app,
        "maintenance",
        vec![DesktopTask {
            id: "data-maintenance".into(),
            state,
            progress,
            started_at,
        }],
    )
}
