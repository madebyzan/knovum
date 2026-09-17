use std::path::Path;

use tauri::Manager;

mod handler;
mod host;

pub(super) fn open(app: &tauri::AppHandle, path: &Path) -> Result<(), String> {
    let owner = app
        .get_webview_window("main")
        .ok_or("The main window is unavailable")?
        .hwnd()
        .map_err(|error| error.to_string())?;
    host::open(path, owner.0 as isize)
}
