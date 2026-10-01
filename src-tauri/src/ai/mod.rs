pub mod capabilities;
mod channel;
mod commands;
pub mod contracts;
pub mod executions;
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
        .invoke_handler(tauri::generate_handler![
            crate::ai::commands::text::cancel_job,
            crate::ai::commands::text::generate_text,
            crate::ai::commands::text::generate_text_stream,
            crate::ai::queries::catalog::detect_agent_installations,
            crate::ai::queries::catalog::list_agent_descriptors,
            crate::ai::queries::catalog::list_provider_descriptors,
            crate::ai::commands::text::transform_editor_text,
            crate::ai::commands::text::transform_editor_text_stream,
            crate::ai::commands::text::translate_text,
            crate::ai::commands::text::translate_text_stream,
            crate::ai::settings::validate_model_settings,
            crate::ai::usage::get_token_usage_report,
        ])
        .build()
}
