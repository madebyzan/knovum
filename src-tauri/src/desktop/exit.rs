use crate::app_error::AppError;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

static APPROVED: AtomicBool = AtomicBool::new(false);
static READY: AtomicBool = AtomicBool::new(false);
static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);
static PENDING: Mutex<Option<ExitRequest>> = Mutex::new(None);

struct ExitRequest {
    id: u64,
    acknowledged_at: Instant,
    showing_error: bool,
}

#[derive(Clone, serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppExitRequest {
    pub request_id: u64,
}

pub(crate) fn take_approval() -> bool {
    APPROVED.swap(false, Ordering::SeqCst)
}
pub(crate) fn revoke_approval() {
    APPROVED.store(false, Ordering::SeqCst);
}
pub(crate) fn renderer_loading() {
    READY.store(false, Ordering::SeqCst);
}

pub(crate) fn restart<R: Runtime>(app: &AppHandle<R>) -> ! {
    APPROVED.store(true, Ordering::SeqCst);
    app.restart()
}

pub(crate) fn approve<R: Runtime>(app: &AppHandle<R>) {
    APPROVED.store(true, Ordering::SeqCst);
    app.exit(0);
}

#[tauri::command]
pub(crate) fn acknowledge_app_exit_request(request_id: u64) -> Result<(), AppError> {
    let mut pending = PENDING.lock().map_err(|_| state_unavailable())?;
    let request = pending
        .as_mut()
        .filter(|request| request.id == request_id && !request.showing_error)
        .ok_or_else(stale_request)?;
    request.acknowledged_at = Instant::now();
    Ok(())
}

#[tauri::command]
pub(crate) fn decide_app_exit_request<R: Runtime>(
    app: AppHandle<R>,
    request_id: u64,
    approved: bool,
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
    if approved {
        approve(&app);
    }
    Ok(())
}

#[tauri::command]
pub(crate) fn mark_exit_coordinator_ready<R: Runtime>(app: AppHandle<R>) -> Result<(), AppError> {
    READY.store(true, Ordering::SeqCst);
    let id = PENDING
        .lock()
        .map_err(|_| state_unavailable())?
        .as_ref()
        .filter(|request| !request.showing_error)
        .map(|request| request.id);
    if let Some(id) = id {
        emit_request(&app, id);
    }
    Ok(())
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
        *pending = Some(ExitRequest {
            id,
            acknowledged_at: Instant::now(),
            showing_error: false,
        });
    }
    if READY.load(Ordering::SeqCst) {
        emit_request(app, id);
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
                request.acknowledged_at.elapsed() >= Duration::from_secs(15)
            };
            if expired {
                show_unresponsive(&app, id);
                return;
            }
        }
    });
}

fn emit_request<R: Runtime>(app: &AppHandle<R>, id: u64) {
    if let Err(error) = app.emit(
        "knovum://app-exit-requested",
        AppExitRequest { request_id: id },
    ) {
        log::error!("failed to deliver exit request: {error}");
        show_unresponsive(app, id);
    }
}

fn show_unresponsive<R: Runtime>(app: &AppHandle<R>, id: u64) {
    {
        let Ok(mut pending) = PENDING.lock() else {
            show_state_error(app);
            return;
        };
        let Some(request) = pending.as_mut().filter(|request| request.id == id) else {
            return;
        };
        if request.showing_error {
            return;
        }
        request.showing_error = true;
    }
    let handle = app.clone();
    app.dialog().message("Knovum is not responding to the quit request. Unsaved edits may be lost if you quit now.")
        .title("Knovum — Unable to finish quitting")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom("Quit anyway".into(), "Keep open".into()))
        .show(move |quit| {
            let Ok(mut pending) = PENDING.lock() else { return; };
            if pending.as_ref().is_none_or(|request| request.id != id) { return; }
            *pending = None;
            drop(pending);
            if quit { approve(&handle); }
        });
}

fn show_state_error<R: Runtime>(app: &AppHandle<R>) {
    log::error!("application exit state is unavailable");
    let handle = app.clone();
    app.dialog()
        .message("Knovum could not coordinate quitting. Unsaved edits may be lost if you quit now.")
        .title("Knovum — Unable to finish quitting")
        .kind(MessageDialogKind::Error)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Quit anyway".into(),
            "Keep open".into(),
        ))
        .show(move |quit| {
            if quit {
                approve(&handle);
            }
        });
}

fn state_unavailable() -> AppError {
    AppError::new(
        "app_exit_state_unavailable",
        "Application exit state is unavailable",
    )
}

fn stale_request() -> AppError {
    AppError::new(
        "app_exit_request_stale",
        "Application exit request is no longer pending",
    )
}
