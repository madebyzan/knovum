use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
mod macos;
pub(crate) mod thumbnail;
#[cfg(target_os = "windows")]
mod windows;

/// A synchronous command runs on the main thread, which is exactly where the preview panel has to
/// be driven from, so it could never wait for its own dispatch. `async` moves the request onto the
/// runtime and leaves the main thread free to answer it.
pub(crate) async fn open(app: tauri::AppHandle, path: PathBuf) -> Result<(), String> {
    crate::blocking::run(move || preview(&app, &path)).await
}

#[cfg(target_os = "macos")]
fn preview(app: &tauri::AppHandle, path: &Path) -> Result<(), String> {
    macos::toggle(app, path)
}

#[cfg(target_os = "windows")]
fn preview(app: &tauri::AppHandle, path: &Path) -> Result<(), String> {
    windows::open(app, path)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn preview(app: &tauri::AppHandle, path: &Path) -> Result<(), String> {
    let _ = (app, path);
    Err("Native document preview is unavailable on this platform".into())
}

#[cfg(test)]
mod tests;
