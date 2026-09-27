use crate::command_error::CommandError;
pub(crate) mod file_drop;
#[cfg(target_os = "macos")]
mod menu;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod tray;

use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "macos")]
use tauri::menu::{Menu, MenuEvent};
use tauri::Emitter;
use tauri::{plugin::TauriPlugin, AppHandle, RunEvent, Runtime, Window, WindowEvent, Wry};

const MAIN_WINDOW_LABEL: &str = "main";
const APP_EXIT_REQUESTED_EVENT: &str = "knovum://app-exit-requested";
static APP_EXIT_APPROVED: AtomicBool = AtomicBool::new(false);
static APP_EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);
static EXIT_COORDINATOR_READY: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "macos")]
pub fn build_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    menu::build(app, "en")
}

#[cfg(target_os = "macos")]
pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    menu::handle_event(app, event);
}

#[tauri::command]
pub fn set_locale<R: Runtime>(app: AppHandle<R>, locale: String) -> Result<(), CommandError> {
    let locale = match locale.as_str() {
        "en" => "en",
        "zh-CN" => "zh-CN",
        _ => return Err("unsupported system locale".into()),
    };

    #[cfg(target_os = "macos")]
    menu::rebuild(&app, locale).map_err(|error| error.to_string())?;

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    tray::rebuild(&app, locale).map_err(|error| error.to_string())?;

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let _ = (&app, locale);

    Ok(())
}

#[tauri::command]
pub fn set_workspace_menu_state<R: Runtime>(
    app: AppHandle<R>,
    workspace_active: bool,
    can_close: bool,
    can_reopen: bool,
) -> Result<(), CommandError> {
    #[cfg(target_os = "macos")]
    menu::set_workspace_state(&app, workspace_active, can_close, can_reopen)
        .map_err(|error| error.to_string())?;

    #[cfg(not(target_os = "macos"))]
    let _ = (app, workspace_active, can_close, can_reopen);

    Ok(())
}

#[tauri::command]
pub fn reveal_main_window<R: Runtime>(app: AppHandle<R>) -> Result<(), CommandError> {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    tray::ensure_installed(&app, "en").map_err(|error| error.to_string())?;
    Ok((crate::app_window::focus_main(&app))?)
}

#[tauri::command]
pub fn decide_app_exit_request<R: Runtime>(app: AppHandle<R>, approved: bool) {
    APP_EXIT_REQUESTED.store(false, Ordering::SeqCst);
    if approved {
        APP_EXIT_APPROVED.store(true, Ordering::SeqCst);
        app.exit(0);
    }
}

#[tauri::command]
pub fn mark_exit_coordinator_ready<R: Runtime>(app: AppHandle<R>) {
    EXIT_COORDINATOR_READY.store(true, Ordering::SeqCst);
    if APP_EXIT_REQUESTED.load(Ordering::SeqCst) {
        emit_app_exit_request(&app);
    }
}

pub fn plugin() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("desktop")
        .invoke_handler(tauri::generate_handler![
            set_locale,
            set_workspace_menu_state,
            reveal_main_window,
            mark_exit_coordinator_ready,
            decide_app_exit_request
        ])
        .build()
}

pub fn handle_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if window.label() != MAIN_WINDOW_LABEL {
        return;
    }

    #[cfg(target_os = "macos")]
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
    }

    #[cfg(target_os = "windows")]
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    if matches!(event, WindowEvent::CloseRequested { .. }) {
        window.app_handle().exit(0);
    }
}

pub fn handle_run_event<R: Runtime>(app: &AppHandle<R>, event: &RunEvent) {
    if let RunEvent::ExitRequested { api, .. } = event {
        if APP_EXIT_APPROVED.swap(false, Ordering::SeqCst) {
            return;
        }
        api.prevent_exit();
        request_app_exit(app);
        return;
    }

    #[cfg(target_os = "macos")]
    if let RunEvent::Reopen {
        has_visible_windows,
        ..
    } = event
    {
        if !has_visible_windows {
            if let Err(error) = crate::app_window::focus_main(app) {
                log::error!("failed to reopen main window from Dock: {error}");
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    let _ = (app, event);
}

pub(crate) fn request_app_exit<R: Runtime>(app: &AppHandle<R>) {
    if APP_EXIT_REQUESTED.swap(true, Ordering::SeqCst) {
        return;
    }
    if EXIT_COORDINATOR_READY.load(Ordering::SeqCst) {
        emit_app_exit_request(app);
    }
}

fn emit_app_exit_request<R: Runtime>(app: &AppHandle<R>) {
    if let Err(error) = app.emit(APP_EXIT_REQUESTED_EVENT, ()) {
        APP_EXIT_REQUESTED.store(false, Ordering::SeqCst);
        log::error!("failed to request coordinated application exit: {error}");
    }
}
