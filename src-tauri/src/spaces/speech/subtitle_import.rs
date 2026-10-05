use crate::{app_error::AppError, file_inspection, spaces::assets};
use std::{fs, path::Path};
use tauri::AppHandle;

#[tauri::command]
pub async fn read_dubbing_subtitles(
    app: AppHandle,
    source_path: String,
) -> Result<String, AppError> {
    let source = Path::new(&source_path);
    assets::authorize_source(&app, source)?;
    let source = source.to_owned();
    Ok(crate::blocking::run(move || -> Result<_, String> {
        let metadata = fs::metadata(&source).map_err(|error| error.to_string())?;
        if !metadata.is_file()
            || metadata.len() > 4 * 1024 * 1024
            || !source
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| matches!(value.to_ascii_lowercase().as_str(), "srt" | "vtt"))
        {
            return Err("Select an SRT or WebVTT subtitle file up to 4 MiB".into());
        }
        let revision = file_inspection::content_revision(&source)?;
        let text = fs::read_to_string(&source).map_err(|error| error.to_string())?;
        file_inspection::verify_content_revision(&source, &revision)?;
        Ok(text)
    })
    .await?)
}
