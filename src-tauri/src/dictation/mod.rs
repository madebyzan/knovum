mod actor;
mod completion;
mod contracts;
mod insertion;
mod lifecycle;
mod progress;
mod recorder;
mod service;
mod transcription;

use std::sync::Arc;

use tauri::{plugin::TauriPlugin, Manager, State, Wry};
use tauri_plugin_global_shortcut::ShortcutState;

use crate::app_error::{AppError, AppResult};
use contracts::DictationSnapshot;
use service::{DictationAction, DictationService};

const SHORTCUT: &str = "CommandOrControl+Shift+Space";

pub(crate) fn global_shortcut_plugin() -> TauriPlugin<Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_shortcut(SHORTCUT)
        .expect("dictation shortcut is a static valid accelerator")
        .with_handler(|app, _, event| {
            let action = match event.state {
                ShortcutState::Pressed => DictationAction::Press,
                ShortcutState::Released => DictationAction::Release,
            };
            if let Some(service) = app.try_state::<Arc<DictationService>>() {
                service.dispatch(action);
            }
        })
        .build()
}

pub(crate) fn plugin() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("dictation")
        .setup(|app, _api| {
            app.manage(Arc::new(DictationService::start(app.clone())));
            Ok(())
        })
        .on_event(|app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                if let Some(service) = app.try_state::<Arc<DictationService>>() {
                    service.dispatch(DictationAction::Shutdown);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            cancel,
            dismiss_transcription,
            copy_transcription
        ])
        .build()
}

#[tauri::command]
fn get_state(service: State<'_, Arc<DictationService>>) -> AppResult<DictationSnapshot> {
    service.snapshot().map_err(AppError::from)
}

#[tauri::command]
fn cancel(service: State<'_, Arc<DictationService>>) {
    service.dispatch(DictationAction::Cancel);
}

#[tauri::command]
fn dismiss_transcription(service: State<'_, Arc<DictationService>>) {
    service.dispatch(DictationAction::Dismiss);
}

#[tauri::command]
async fn copy_transcription(service: State<'_, Arc<DictationService>>) -> AppResult<()> {
    let text = service
        .snapshot()
        .map_err(AppError::from)?
        .text
        .ok_or_else(|| AppError::invalid("No dictation result is available"))?;
    crate::blocking::run(move || insertion::copy_text(text))
        .await
        .map_err(AppError::from)
}
