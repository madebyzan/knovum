mod agent_runtime;
mod ai;
mod app_builder;
pub mod app_error;
mod app_resources;
mod app_window;
mod blocking;
mod clock;
pub mod contract_codegen;
mod desktop;
#[cfg(knovum_supported_desktop)]
mod dictation;
#[cfg(not(knovum_supported_desktop))]
mod dictation_unsupported;
mod documents;
mod extensions;
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
mod subprocess;
mod system_font;
mod system_preview;
mod updater;
mod utf16;
mod web_capture;

pub fn run() {
    app_builder::run();
}
