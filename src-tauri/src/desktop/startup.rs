use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

static GENERATION: AtomicU64 = AtomicU64::new(0);
static SETTLED: AtomicBool = AtomicBool::new(false);
static SHOWING_ERROR: AtomicBool = AtomicBool::new(false);

pub(crate) fn begin<R: Runtime>(app: &AppHandle<R>) {
    SETTLED.store(false, Ordering::SeqCst);
    let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            if GENERATION.load(Ordering::SeqCst) != generation || SETTLED.load(Ordering::SeqCst) {
                return;
            }
            if crate::backup::is_running(&app) {
                continue;
            }
            fail(
                &app,
                "The application did not reach a usable state within 60 seconds.",
            );
            return;
        }
    });
}

pub(crate) fn settle() {
    SETTLED.store(true, Ordering::SeqCst);
}

pub(crate) fn fail<R: Runtime>(app: &AppHandle<R>, reason: &str) {
    if SETTLED.swap(true, Ordering::SeqCst) || SHOWING_ERROR.swap(true, Ordering::SeqCst) {
        return;
    }
    log::error!("desktop startup failed: {reason}");
    let handle = app.clone();
    app.dialog()
        .message(format!("Knovum could not start.\n\n{reason}"))
        .title("Knovum — Startup failed")
        .kind(MessageDialogKind::Error)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Retry".into(),
            "Quit".into(),
        ))
        .show(move |retry| {
            SHOWING_ERROR.store(false, Ordering::SeqCst);
            if !retry {
                super::exit::approve(&handle);
                return;
            }
            if let Some(window) = handle.get_webview_window("main") {
                begin(&handle);
                if let Err(error) = window.eval("location.reload()") {
                    fail(&handle, &error.to_string());
                }
            } else {
                begin(&handle);
                fail(&handle, "The main application window is unavailable.");
            }
        });
}

#[tauri::command]
pub(crate) fn fail_startup<R: Runtime>(
    app: AppHandle<R>,
    reason: String,
) -> Result<(), crate::foundation::app_error::AppError> {
    if reason.is_empty() || reason.len() > 4096 {
        return Err("invalid startup failure diagnostic".into());
    }
    fail(&app, &reason);
    Ok(())
}
