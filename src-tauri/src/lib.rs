mod agent_runtime;
mod ai;
mod app_builder;
mod app_resources;
mod app_window;
mod blocking;
mod clock;
mod command_error;
pub mod contract_codegen;
mod desktop;
mod documents;
mod external_agents;
mod file_inspection;
mod images;
mod job_registry;
mod local_resources;
mod media;
mod public_network;
mod secrets;
mod spaces;
mod sqlite;
mod system_font;
mod system_preview;
mod updater;
mod utf16;
mod web_capture;

pub fn run() {
    app_builder::run();
}
