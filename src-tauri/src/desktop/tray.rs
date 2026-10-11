use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Runtime,
};

use super::locale::labels;
use crate::preferences::LocalePreference;

const TRAY_ID: &str = "knovum-main-tray";

fn build_menu<R: Runtime>(app: &AppHandle<R>, locale: LocalePreference) -> tauri::Result<Menu<R>> {
    let labels = labels(locale).map_err(std::io::Error::other)?.tray;
    let show = MenuItem::with_id(app, "show", labels.show, true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", labels.quit, true, None::<&str>)?;
    Menu::with_items(app, &[&show, &separator, &quit])
}

pub fn install<R: Runtime>(app: &AppHandle<R>, locale: LocalePreference) -> tauri::Result<()> {
    let menu = build_menu(app, locale)?;
    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Knovum")
        .menu(&menu)
        .show_menu_on_left_click(cfg!(target_os = "macos"))
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "quit" => super::request_app_exit(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                #[cfg(target_os = "windows")]
                show_main_window(tray.app_handle());

                #[cfg(target_os = "macos")]
                let _ = tray;
            }
        });

    let tray_icon = load_tray_icon().or_else(|| app.default_window_icon().cloned());

    if let Some(icon) = tray_icon {
        tray = tray.icon(icon);
        #[cfg(target_os = "macos")]
        {
            tray = tray.icon_as_template(true);
        }
    }
    tray.build(app)?;
    Ok(())
}

pub fn rebuild<R: Runtime>(app: &AppHandle<R>, locale: LocalePreference) -> tauri::Result<()> {
    let menu = build_menu(app, locale)?;
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(menu))?;
        return Ok(());
    }
    install(app, locale)
}

pub fn ensure_installed<R: Runtime>(
    app: &AppHandle<R>,
    locale: LocalePreference,
) -> tauri::Result<()> {
    if app.tray_by_id(TRAY_ID).is_none() {
        install(app, locale)?;
    }
    Ok(())
}

fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Err(error) = crate::app_window::focus_main(app) {
        log::error!("failed to focus main window: {error}");
    }
}

fn load_tray_icon() -> Option<tauri::image::Image<'static>> {
    let img = match image::load_from_memory(include_bytes!("../../icons/tray_icon.png")) {
        Ok(img) => img,
        Err(error) => {
            log::warn!("failed to load tray icon: {error}");
            return None;
        }
    };
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Some(tauri::image::Image::new_owned(
        rgba.into_raw(),
        width,
        height,
    ))
}
