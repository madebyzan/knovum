mod agent_runtime;
mod ai;
mod app;
mod backup;
pub mod contract_codegen;
mod desktop;
mod dictation;
mod documents;
mod extensions;
mod external_agents;
mod file_inspection;
pub mod foundation;
mod images;
mod local_resources;
mod media;
mod preferences;
mod secrets;
mod spaces;
mod sqlite;
mod web_capture;

pub fn run() {
    documents::generation::worker::run_if_requested();
    app::run();
}
