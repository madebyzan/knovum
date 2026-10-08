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

use tauri::{plugin::TauriPlugin, Wry};

pub fn plugin() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("ai")
        .setup(|app, _api| {
            crate::external_agents::task::discard_interrupted(app)
                .map_err(std::io::Error::other)?;
            tauri::async_runtime::block_on(usage::initialize(app))
                .map_err(std::io::Error::other)?;
            Ok(())
        })
        .on_event(|_, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                tauri::async_runtime::block_on(local_runtime::shutdown());
            }
        })
        .invoke_handler(tauri::generate_handler![
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
