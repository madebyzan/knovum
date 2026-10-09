use super::TaskHub;
use crate::desktop_task::{DesktopTask, DesktopTaskState};
use std::sync::atomic::Ordering;
use tauri::{
    window::{ProgressBarState, ProgressBarStatus},
    AppHandle, Manager, Runtime,
};
use tauri_plugin_notification::NotificationExt;

pub(super) fn progress<'a, R: Runtime>(
    app: &AppHandle<R>,
    tasks: impl Iterator<Item = &'a DesktopTask>,
) -> Result<(), String> {
    let tasks: Vec<_> = tasks.collect();
    let Some(window) = app.get_webview_window("main") else {
        return Err("main window is unavailable".into());
    };
    let (status, progress) = if tasks.is_empty() {
        (ProgressBarStatus::None, None)
    } else if tasks.iter().any(|task| task.progress.is_none()) {
        (ProgressBarStatus::Indeterminate, None)
    } else {
        let sum: u64 = tasks
            .iter()
            .filter_map(|task| task.progress.map(u64::from))
            .sum();
        (ProgressBarStatus::Normal, Some(sum / tasks.len() as u64))
    };
    window
        .set_progress_bar(ProgressBarState {
            status: Some(status),
            progress,
        })
        .map_err(|error| error.to_string())
}

pub(super) fn notify<R: Runtime>(app: &AppHandle<R>, hub: &TaskHub, task: &DesktopTask) {
    if !hub.notifications.load(Ordering::SeqCst)
        || !matches!(
            task.state,
            DesktopTaskState::Succeeded | DesktopTaskState::Failed
        )
    {
        return;
    }
    let Ok(now) = crate::clock::unix_time_millis() else {
        return;
    };
    if now.saturating_sub(task.started_at) < 10_000 {
        return;
    }
    let background = app.get_webview_window("main").is_some_and(|window| {
        window.is_visible().is_ok_and(|visible| !visible)
            || window.is_focused().is_ok_and(|focused| !focused)
    });
    if !background {
        return;
    }
    let chinese = hub.chinese.load(Ordering::SeqCst);
    let body = match (chinese, task.state == DesktopTaskState::Failed) {
        (true, true) => "任务失败。打开任务中心查看详情。",
        (true, false) => "任务已完成。",
        (false, true) => "Task failed. Open Task Center for details.",
        (false, false) => "Task completed.",
    };
    // Notification copy never contains prompts, output, credentials, or producer diagnostics.
    if let Err(error) = app
        .notification()
        .builder()
        .title("Knovum")
        .body(body)
        .show()
    {
        log::warn!("desktop notification unavailable: {error}");
    }
}
