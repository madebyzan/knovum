pub mod capabilities;
mod channel;
mod commands;
pub mod contracts;
pub mod executions;
pub(crate) mod local_runtime;
pub mod model_turn;
pub mod models;
pub mod providers;
mod queries;
pub mod settings;
pub mod tasks;
pub mod transport;
pub mod usage;

use tauri::{plugin::TauriPlugin, AppHandle, Manager, Wry};

#[derive(Default)]
struct AiInitialization(tokio::sync::OnceCell<Result<(), String>>);

pub(crate) async fn await_runtime(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AiInitialization>();
    state
        .0
        .get_or_init(|| async {
            crate::backup::ready(app).await?;
            usage::initialize(app).await
        })
        .await
        .clone()
}

#[tauri::command]
async fn await_ai_runtime(app: AppHandle) -> Result<(), crate::foundation::app_error::AppError> {
    Ok(await_runtime(&app).await?)
}

pub fn plugin() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("ai")
        .setup(|app, _api| {
            app.manage(AiInitialization::default());
            Ok(())
        })
        .on_event(|app, event| {
            if matches!(event, tauri::RunEvent::Ready) {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = await_runtime(&app).await {
                        log::error!("AI runtime initialization failed: {error}");
                    }
                });
            }
            if matches!(event, tauri::RunEvent::Exit) {
                tauri::async_runtime::block_on(local_runtime::shutdown());
            }
        })
        .invoke_handler(tauri::generate_handler![
            await_ai_runtime,
            crate::ai::commands::text::cancel_job,
            crate::ai::queries::catalog::detect_external_agent_installations,
            crate::ai::queries::catalog::list_external_agent_descriptors,
            crate::ai::queries::catalog::list_provider_descriptors,
            crate::ai::commands::text::transform_editor_text_stream,
            crate::ai::commands::text::translate_text_stream,
            crate::ai::settings::validate_model_settings,
            crate::ai::usage::get_token_usage_report,
        ])
        .build()
}
