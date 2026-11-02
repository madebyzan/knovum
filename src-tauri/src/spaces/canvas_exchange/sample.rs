use tauri::{path::BaseDirectory, AppHandle, Manager};
use tauri_plugin_fs::FsExt;

use crate::{foundation::app_error::AppError, preferences::LocalePreference};

/// Where the sample Space for a language is bundled.
fn relative_path(locale: LocalePreference) -> Result<String, String> {
    let tag = serde_json::to_value(locale).map_err(|error| error.to_string())?;
    let tag = tag
        .as_str()
        .ok_or("A locale serializes to its language tag")?;
    Ok(format!("resources/sample-space/{tag}.canvas"))
}

/// The sample Space is an ordinary JSON Canvas document, imported the way a person's would be. A
/// document reaches that import only through a path the user granted; here the application grants
/// its own read-only resource, and the locale's closed type keeps the path inside that directory.
#[tauri::command]
pub(crate) fn sample_space_source(
    app: AppHandle,
    locale: LocalePreference,
) -> Result<String, AppError> {
    let source = app
        .path()
        .resolve(relative_path(locale)?, BaseDirectory::Resource)
        .map_err(|error| error.to_string())?;
    app.fs_scope()
        .allow_file(&source)
        .map_err(|error| error.to_string())?;
    Ok(source.to_string_lossy().into_owned())
}

#[cfg(test)]
#[path = "tests/sample.rs"]
mod tests;
