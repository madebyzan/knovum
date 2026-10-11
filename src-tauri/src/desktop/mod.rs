use crate::app_error::AppError;
mod exit;
pub(crate) mod file_drop;
pub(crate) use exit::AppExitRequest;
mod editing;
mod locale;
pub(crate) use editing::EditingCommand;
#[cfg(target_os = "macos")]
mod menu;
mod startup;
pub(crate) mod tasks;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod tray;
mod window_close;

#[cfg(target_os = "macos")]
use tauri::menu::{Menu, MenuEvent};
use tauri::{plugin::TauriPlugin, AppHandle, Manager, RunEvent, Runtime, Window, WindowEvent, Wry};

const MAIN_WINDOW_LABEL: &str = "main";
#[cfg(target_os = "macos")]
pub fn build_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    menu::build(app, crate::preferences::LocalePreference::English)
}

#[cfg(target_os = "macos")]
pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    menu::handle_event(app, event);
}

#[tauri::command]
pub fn set_locale<R: Runtime>(
    app: AppHandle<R>,
    locale: crate::preferences::LocalePreference,
) -> Result<(), AppError> {
    locale::labels(locale)?;
    tasks::set_locale(&app, locale)?;
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
) -> Result<(), AppError> {
    #[cfg(target_os = "macos")]
    menu::set_workspace_state(&app, workspace_active, can_close, can_reopen)
        .map_err(|error| error.to_string())?;

    #[cfg(not(target_os = "macos"))]
    let _ = (app, workspace_active, can_close, can_reopen);

    Ok(())
}

#[tauri::command]
pub fn reveal_main_window<R: Runtime>(app: AppHandle<R>) -> Result<(), AppError> {
    let reveal = || -> Result<(), String> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        tray::ensure_installed(&app, crate::preferences::LocalePreference::English)
            .map_err(|error| error.to_string())?;
        crate::app_window::focus_main(&app)
    };
    match reveal() {
        Ok(()) => {
            startup::settle();
            Ok(())
        }
        Err(error) => {
            startup::fail(
                &app,
                &format!("The main window could not be revealed: {error}"),
            );
            Err(AppError::new("main_window_reveal_failed", error))
        }
    }
}

pub fn plugin() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("desktop")
        .invoke_handler(tauri::generate_handler![
            set_locale,
            set_workspace_menu_state,
            reveal_main_window,
            exit::mark_exit_coordinator_ready,
            exit::acknowledge_app_exit_request,
            exit::decide_app_exit_request,
            startup::report_startup_failure,
            tasks::project_session_tasks,
            tasks::set_task_notifications,
            editing::perform_editing,
            window_close::begin_window_close,
            window_close::acknowledge_window_close,
            window_close::complete_window_close
        ])
        .build()
}

pub fn handle_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if window.label() != MAIN_WINDOW_LABEL {
        return;
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        window_close::request(window.app_handle());
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    if matches!(event, WindowEvent::CloseRequested { .. }) {
        window.app_handle().exit(0);
    }
}

pub fn handle_run_event<R: Runtime>(app: &AppHandle<R>, event: &RunEvent) {
    if let RunEvent::ExitRequested { api, code, .. } = event {
        if app
            .try_state::<crate::updater::UpdateService>()
            .is_some_and(|service| service.blocks_exit())
        {
            exit::revoke_approval();
            if *code != Some(tauri::RESTART_EXIT_CODE) {
                api.prevent_exit();
            }
            return;
        }
        if exit::take_approval() {
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
    exit::request(app);
}

pub(crate) fn install(app: &AppHandle) {
    tasks::install(app);
    startup::begin(app);
}

pub(crate) fn renderer_loading<R: Runtime>(app: &AppHandle<R>) {
    exit::renderer_loading();
    startup::begin(app);
}

pub(crate) fn restart_for_maintenance<R: Runtime>(app: &AppHandle<R>) -> ! {
    exit::restart(app)
}

pub(crate) fn set_maintenance_progress<R: Runtime>(
    app: &AppHandle<R>,
    progress: Option<u8>,
) -> Result<(), String> {
    tasks::maintenance(app, progress, None)
}

pub(crate) fn finish_maintenance_progress<R: Runtime>(
    app: &AppHandle<R>,
    succeeded: bool,
) -> Result<(), String> {
    tasks::maintenance(app, Some(100), Some(succeeded))
}

#[cfg(test)]
#[path = "tests/capabilities.rs"]
mod capability_tests;
