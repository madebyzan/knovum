use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
mod macos;
pub(crate) mod thumbnail;

/// Shows one file the way the platform shows content to look at. macOS has a system preview panel,
/// Quick Look, so the file opens there without leaving Knovum. Windows and Linux have no equivalent
/// for arbitrary content — Windows Preview Handlers serve the Explorer and Outlook reading panes and
/// none ship for pictures — so the file opens in its associated application, which is how those
/// platforms show content.
///
/// A synchronous command runs on the main thread, which is exactly where the preview panel has to
/// be driven from, so it could never wait for its own dispatch. `async` moves the request onto the
/// runtime and leaves the main thread free to answer it.
pub(crate) async fn open(app: tauri::AppHandle, path: PathBuf) -> Result<(), String> {
    crate::foundation::blocking::run(move || show(&app, &path)).await
}

#[cfg(target_os = "macos")]
fn show(app: &tauri::AppHandle, path: &Path) -> Result<(), String> {
    macos::toggle(app, path)
}

#[cfg(not(target_os = "macos"))]
fn show(app: &tauri::AppHandle, path: &Path) -> Result<(), String> {
    super::associated_app::open(app, path)
}

#[cfg(test)]
mod tests;
