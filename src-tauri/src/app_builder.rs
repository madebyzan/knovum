use tauri::{Builder, Wry};

use crate::{ai, app_resources, app_window, desktop, extensions, secrets, spaces, updater};

pub(crate) fn run() {
    let builder = platform_builder(base_builder());
    let app = finish(builder);
    app.run(|app, event| desktop::handle_run_event(app, &event));
}

fn base_builder() -> Builder<Wry> {
    let builder = Builder::default()
        .invoke_handler(tauri::generate_handler![
            updater::get_update_state,
            updater::check_for_update,
            updater::download::download_update,
            updater::download::cancel_update_download,
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
        .plugin(extensions::plugin())
        .plugin(spaces::plugin());
    dictation_builder(builder)
}

#[cfg(knovum_supported_desktop)]
fn dictation_builder(builder: Builder<Wry>) -> Builder<Wry> {
    builder
        .plugin(crate::dictation::plugin())
        .plugin(crate::dictation::global_shortcut_plugin())
}

#[cfg(not(knovum_supported_desktop))]
fn dictation_builder(builder: Builder<Wry>) -> Builder<Wry> {
    builder.plugin(crate::dictation_unsupported::plugin())
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
            updater::setup(app.handle())?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Tauri application")
}
