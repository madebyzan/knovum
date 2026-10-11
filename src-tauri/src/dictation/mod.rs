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
mod transcription;

#[cfg(knovum_supported_desktop)]
mod runtime {
    use std::sync::Arc;

    use tauri::{plugin::TauriPlugin, Manager, State, Wry};
    use tauri_plugin_global_shortcut::ShortcutState;

    use super::{
        contracts::DictationSnapshot,
        engine::{DictationAction, DictationEngine},
        insertion,
    };
    use crate::foundation::app_error::{AppError, AppResult};

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
                copy_transcription
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
}

#[cfg(knovum_supported_desktop)]
pub(crate) use runtime::{global_shortcut_plugin, plugin};
