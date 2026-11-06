pub(crate) mod contracts;
#[cfg(not(knovum_supported_desktop))]
pub(crate) mod unsupported;

#[cfg(knovum_supported_desktop)]
mod actor;
#[cfg(knovum_supported_desktop)]
mod completion;
#[cfg(knovum_supported_desktop)]
mod engine;
#[cfg(knovum_supported_desktop)]
mod insertion;
#[cfg(knovum_supported_desktop)]
mod lifecycle;
#[cfg(knovum_supported_desktop)]
mod progress;
#[cfg(knovum_supported_desktop)]
mod recorder;
#[cfg(knovum_supported_desktop)]
pub(crate) mod shortcut;
#[cfg(knovum_supported_desktop)]
mod transcription;

#[cfg(knovum_supported_desktop)]
mod runtime {
    use std::sync::Arc;

    use tauri::{plugin::TauriPlugin, AppHandle, Manager, Runtime, State, Wry};
    use tauri_plugin_global_shortcut::ShortcutState;

    use super::{
        contracts::{DictationShortcutStatus, DictationSnapshot},
        engine::{DictationAction, DictationEngine},
        insertion,
        shortcut::{DictationShortcutManager, DEFAULT_DICTATION_SHORTCUT},
    };
    use crate::foundation::app_error::{AppError, AppResult};

    pub(crate) fn global_shortcut_plugin() -> TauriPlugin<Wry> {
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app, _, event| {
                let action = match event.state {
                    ShortcutState::Pressed => DictationAction::Press,
                    ShortcutState::Released => DictationAction::Release,
                };
                if let Some(engine) = app.try_state::<Arc<DictationEngine>>() {
                    engine.dispatch(action);
                }
            })
            .build()
    }

    pub(crate) fn plugin() -> TauriPlugin<Wry> {
        tauri::plugin::Builder::new("dictation")
            .setup(|app, _api| {
                app.manage(Arc::new(DictationEngine::start(app.clone())));
                let manager = Arc::new(DictationShortcutManager::new());
                manager.register_shortcut(app, DEFAULT_DICTATION_SHORTCUT);
                app.manage(manager);
                Ok(())
            })
            .on_event(|app, event| {
                if matches!(event, tauri::RunEvent::Exit) {
                    if let Some(engine) = app.try_state::<Arc<DictationEngine>>() {
                        engine.dispatch(DictationAction::Shutdown);
                    }
                }
            })
            .invoke_handler(tauri::generate_handler![
                get_state,
                cancel,
                dismiss_transcription,
                copy_transcription,
                get_dictation_shortcut,
                set_dictation_shortcut,
                reset_dictation_shortcut
            ])
            .build()
    }

    #[tauri::command]
    fn get_state(engine: State<'_, Arc<DictationEngine>>) -> AppResult<DictationSnapshot> {
        engine.snapshot().map_err(AppError::from)
    }

    #[tauri::command]
    fn cancel(engine: State<'_, Arc<DictationEngine>>) {
        engine.dispatch(DictationAction::Cancel);
    }

    #[tauri::command]
    fn dismiss_transcription(engine: State<'_, Arc<DictationEngine>>) {
        engine.dispatch(DictationAction::Dismiss);
    }

    #[tauri::command]
    async fn copy_transcription(engine: State<'_, Arc<DictationEngine>>) -> AppResult<()> {
        let text = engine
            .snapshot()
            .map_err(AppError::from)?
            .text
            .ok_or_else(|| AppError::invalid("No dictation result is available"))?;
        crate::foundation::blocking::run(move || insertion::copy_text(text))
            .await
            .map_err(AppError::from)
    }

    #[tauri::command]
    fn get_dictation_shortcut(
        manager: State<'_, Arc<DictationShortcutManager>>,
    ) -> AppResult<DictationShortcutStatus> {
        Ok(manager.current_status())
    }

    #[tauri::command]
    fn set_dictation_shortcut<R: Runtime>(
        app: AppHandle<R>,
        manager: State<'_, Arc<DictationShortcutManager>>,
        shortcut: String,
    ) -> AppResult<DictationShortcutStatus> {
        if shortcut.trim().is_empty() {
            return Err(AppError::invalid("Shortcut cannot be empty"));
        }
        Ok(manager.register_shortcut(&app, &shortcut))
    }

    #[tauri::command]
    fn reset_dictation_shortcut<R: Runtime>(
        app: AppHandle<R>,
        manager: State<'_, Arc<DictationShortcutManager>>,
    ) -> AppResult<DictationShortcutStatus> {
        Ok(manager.register_shortcut(&app, DEFAULT_DICTATION_SHORTCUT))
    }
}

#[cfg(knovum_supported_desktop)]
pub(crate) use runtime::{global_shortcut_plugin, plugin};
