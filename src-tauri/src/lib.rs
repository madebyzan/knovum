mod ai;
mod app_builder;
mod app_window;
mod blocking;
mod clock;
mod desktop;
mod documents;
mod file_inspection;
mod images;
mod job_registry;
mod local_resources;
mod media;
mod secrets;
mod spaces;
mod sqlite;
mod system_preview;
mod updater;
mod utf16;
mod web_capture;

pub fn run() {
    app_builder::run();
}
