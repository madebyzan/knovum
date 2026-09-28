use tauri::{Builder, Wry};

use crate::{ai, app_resources, app_window, desktop, secrets, spaces, updater};

pub(crate) fn run() {
    let builder = base_builder();
    let builder = release_builder(builder);
    let builder = platform_builder(builder);
    let app = finish(builder);
    app.run(|app, event| desktop::handle_run_event(app, &event));
}

fn base_builder() -> Builder<Wry> {
    Builder::default()
        .invoke_handler(tauri::generate_handler![
            updater::check_for_update,
            updater::get_update_configuration,
            updater::install_update,
        ])
        .on_webview_event(desktop::file_drop::authorize_drop)
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Err(error) = app_window::focus_main(app) {
                log::error!("failed to focus existing instance: {error}");
            }
        }))
        .plugin(log_plugin())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_clipboard::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(window_state_plugin())
        .plugin(desktop::plugin())
        .plugin(app_resources::plugin())
        .plugin(secrets::plugin())
        .plugin(ai::plugin())
        .plugin(spaces::plugin())
}

fn log_plugin() -> tauri::plugin::TauriPlugin<Wry> {
    tauri_plugin_log::Builder::new()
        .max_file_size(2_000_000)
        .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(5))
        .level(log::LevelFilter::Info)
        .build()
}

fn window_state_plugin() -> tauri::plugin::TauriPlugin<Wry> {
    tauri_plugin_window_state::Builder::default()
        .with_state_flags(
            tauri_plugin_window_state::StateFlags::SIZE
                | tauri_plugin_window_state::StateFlags::POSITION
                | tauri_plugin_window_state::StateFlags::MAXIMIZED
                | tauri_plugin_window_state::StateFlags::FULLSCREEN,
        )
        .build()
}

#[cfg(not(debug_assertions))]
fn release_builder(builder: Builder<Wry>) -> Builder<Wry> {
    builder.plugin(tauri_plugin_updater::Builder::new().build())
}

#[cfg(debug_assertions)]
fn release_builder(builder: Builder<Wry>) -> Builder<Wry> {
    builder
}

#[cfg(target_os = "macos")]
fn platform_builder(builder: Builder<Wry>) -> Builder<Wry> {
    builder
        .menu(desktop::build_menu)
        .on_menu_event(desktop::handle_menu_event)
}

#[cfg(not(target_os = "macos"))]
fn platform_builder(builder: Builder<Wry>) -> Builder<Wry> {
    builder
}

#[cfg(feature = "e2e")]
fn e2e_builder(builder: Builder<Wry>) -> Builder<Wry> {
    builder
        .plugin(tauri_plugin_wdio::init())
        .plugin(tauri_plugin_wdio_webdriver::init())
}

#[cfg(not(feature = "e2e"))]
fn e2e_builder(builder: Builder<Wry>) -> Builder<Wry> {
    builder
}

fn finish(builder: Builder<Wry>) -> tauri::App<Wry> {
    e2e_builder(builder)
        .on_window_event(desktop::handle_window_event)
        .setup(|app| {
            secrets::install(app.handle()).map_err(std::io::Error::other)?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Tauri application")
}
