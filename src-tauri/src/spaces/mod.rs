mod action_execution;
pub(crate) mod actions;
mod agent_artifacts;
pub(crate) mod assets;
pub(crate) mod assistant;
pub(crate) mod authorization;
pub(crate) mod automation;
mod canvas_exchange;
mod canvas_view_state;
mod clipboard_content;
mod command_registry;
pub(crate) mod commands;
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
pub(crate) mod runs;
pub(crate) mod structure;
pub(crate) mod workflows;

pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("spaces")
        .setup(|app, _api| {
            runs::start_scheduler(app.clone());
            assistant::recover_tasks(app.clone());
            external_conversations::recover(app.clone());
            Ok(())
        })
        .invoke_handler(command_registry::handler())
        .build()
}
