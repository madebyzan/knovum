use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    time::{Duration, Instant},
};

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::app_error::AppError;

const MAIN_WINDOW_LABEL: &str = "main";
static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);
static PENDING: Mutex<Option<CloseRequest>> = Mutex::new(None);

struct CloseRequest {
    id: u64,
    acknowledged_at: Instant,
    showing_error: bool,
}

pub(crate) fn request<R: Runtime>(app: &AppHandle<R>) {
    let id = NEXT_REQUEST.fetch_add(1, Ordering::SeqCst);
    {
        let Ok(mut pending) = PENDING.lock() else {
            show_state_error(app);
            return;
        };
        if pending.is_some() {
            return;
        }
        *pending = Some(CloseRequest {
            id,
            acknowledged_at: Instant::now(),
            showing_error: false,
        });
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            let expired = {
                let Ok(mut pending) = PENDING.lock() else {
                    show_state_error(&app);
                    return;
                };
                let Some(request) = pending.as_mut().filter(|request| request.id == id) else {
                    return;
                };
                if request.showing_error {
                    return;
                }
                if request.acknowledged_at.elapsed() < Duration::from_secs(15) {
                    false
                } else {
                    request.showing_error = true;
                    true
                }
            };
            if expired {
                show_unresponsive(&app, id);
                return;
            }
        }
    });
}

#[tauri::command]
pub(crate) fn begin_window_close() -> Result<u64, AppError> {
    let mut pending = PENDING.lock().map_err(|_| state_unavailable())?;
    let request = pending
        .as_mut()
        .filter(|request| !request.showing_error)
        .ok_or_else(stale_request)?;
    request.acknowledged_at = Instant::now();
    Ok(request.id)
}

#[tauri::command]
pub(crate) fn acknowledge_window_close(request_id: u64) -> Result<(), AppError> {
    let mut pending = PENDING.lock().map_err(|_| state_unavailable())?;
    let request = pending
        .as_mut()
        .filter(|request| request.id == request_id && !request.showing_error)
        .ok_or_else(stale_request)?;
    request.acknowledged_at = Instant::now();
    Ok(())
}

#[tauri::command]
pub(crate) fn complete_window_close<R: Runtime>(
    app: AppHandle<R>,
    request_id: u64,
    hide: bool,
) -> Result<(), AppError> {
    let mut pending = PENDING.lock().map_err(|_| state_unavailable())?;
    if pending
        .as_ref()
        .is_none_or(|request| request.id != request_id || request.showing_error)
    {
        return Err(stale_request());
    }
    *pending = None;
    drop(pending);
    if !hide {
        return Ok(());
    }
    let window = app.get_webview_window(MAIN_WINDOW_LABEL).ok_or_else(|| {
        AppError::new(
            "window_close_owner_unavailable",
            "Main window is unavailable",
        )
    })?;
    window.hide().map_err(|error| {
        AppError::new(
            "window_close_hide_failed",
            format!("The window could not be hidden: {error}"),
        )
    })
}

fn stale_request() -> AppError {
    AppError::new(
        "window_close_request_stale",
        "Window close request is no longer pending",
    )
}

fn state_unavailable() -> AppError {
    AppError::new(
        "window_close_state_unavailable",
        "Window close state is unavailable",
    )
}

fn show_unresponsive<R: Runtime>(app: &AppHandle<R>, id: u64) {
    app.dialog()
        .message("Window close could not finish. The window has been kept open.")
        .title("Knovum — Unable to close window")
        .kind(MessageDialogKind::Error)
        .buttons(MessageDialogButtons::Ok)
        .show(move |_| clear_request(id));
}

fn show_state_error<R: Runtime>(app: &AppHandle<R>) {
    app.dialog()
        .message("Window close could not start. The window has been kept open.")
        .title("Knovum — Unable to close window")
        .kind(MessageDialogKind::Error)
        .buttons(MessageDialogButtons::Ok)
        .show(|_| {});
}

fn clear_request(id: u64) {
    let Ok(mut pending) = PENDING.lock() else {
        return;
    };
    if pending.as_ref().is_some_and(|request| request.id == id) {
        *pending = None;
    }
}
