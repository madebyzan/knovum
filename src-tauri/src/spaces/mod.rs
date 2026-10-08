mod action_execution;
pub(crate) mod actions;
pub(crate) mod agent_artifacts;
pub(crate) mod assets;
pub(crate) mod authorization;
pub(crate) mod canvas_exchange;
mod canvas_view_state;
mod clipboard_content;
mod command_registry;
pub(crate) mod commands;
pub(crate) mod conversations;
pub(crate) mod db;
pub(crate) mod derivation;
pub(crate) mod external_conversations;
pub(crate) mod knowledge;
pub(crate) mod mcp;
pub(crate) mod nodes;
pub(crate) mod package;
mod previews;
pub(crate) mod queries;
mod recovery;
mod run_control;
mod run_execution;
pub(crate) mod runs;
pub(crate) mod speech;
pub(crate) mod structure;
pub(crate) mod workflows;

pub(crate) use run_control::RunControlReceipt;

pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    use tauri::Manager;
    tauri::plugin::Builder::new("spaces")
        .setup(|app, _api| {
            app.manage(
                conversations::tool_operations::concurrency::ToolInvocationConcurrency::default(),
            );
            derivation::image_operations::discard_interrupted(app)?;
            recovery::start(app.clone());
            Ok(())
        })
        .invoke_handler(command_registry::handler())
        .build()
}
