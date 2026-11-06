use std::path::Path;

/// Opens an authorized file in the application the system associates with it — Photos for a
/// picture on Windows, Preview on macOS — exactly as double-clicking it in the file manager would.
pub(crate) fn open(app: &tauri::AppHandle, file: &Path) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;

    app.opener()
        .open_path(file.to_string_lossy().into_owned(), None::<String>)
        .map_err(|error| error.to_string())
}
