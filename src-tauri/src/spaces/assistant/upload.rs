use crate::app_error::AppError;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri_plugin_fs::FsExt;

const MAX_FILE_BYTES: u64 = 20 * 1024 * 1024;
const MAX_IMAGE_BYTES: u64 = 10 * 1024 * 1024;

#[derive(Clone, Copy)]
pub(super) enum UploadKind {
    Image(&'static str),
    Text,
    Pdf,
    Office,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct InspectedAssistantUpload {
    pub path: String,
    pub source_revision: String,
}

#[tauri::command]
pub async fn inspect_assistant_upload(
    app: tauri::AppHandle,
    path: String,
) -> Result<InspectedAssistantUpload, AppError> {
    crate::blocking::run(move || inspect(&app, &path)).await
}

#[tauri::command]
pub async fn preview_assistant_upload(app: tauri::AppHandle, path: String) -> Result<(), AppError> {
    let source = authorize(&app, &path)?;
    classify(&source)?;
    crate::system_preview::open(app, source).await?;
    Ok(())
}

fn inspect(app: &tauri::AppHandle, path: &str) -> Result<InspectedAssistantUpload, AppError> {
    let source = authorize(app, path)?;
    source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            AppError::new(
                "assistant_upload_invalid",
                "assistant-upload-invalid: filename is not Unicode",
            )
        })?;
    let size = std::fs::metadata(&source)
        .map_err(|error| error.to_string())?
        .len();
    let kind = classify(&source)?;
    enforce_size(kind, size)?;
    match kind {
        UploadKind::Image(media_type) => {
            read_image(&source, media_type)?;
        }
        UploadKind::Text => {
            String::from_utf8(read_bounded(&source, MAX_FILE_BYTES)?).map_err(|error| {
                AppError::new(
                    "assistant_upload_invalid",
                    format!("assistant-upload-invalid: text is not UTF-8: {error}"),
                )
            })?;
        }
        UploadKind::Pdf | UploadKind::Office => {}
    }
    Ok(InspectedAssistantUpload {
        path: source.to_string_lossy().into_owned(),
        source_revision: crate::file_inspection::content_revision(&source)?,
    })
}

pub(super) fn authorize(app: &tauri::AppHandle, path: &str) -> Result<PathBuf, AppError> {
    let source = Path::new(path);
    crate::spaces::assets::authorize_source(app, source).map_err(|error| {
        AppError::new(
            "assistant_upload_unauthorized",
            format!("assistant-upload-unauthorized: {error}"),
        )
    })?;
    if !source.is_file() {
        return Err(AppError::new(
            "assistant_upload_invalid",
            "assistant-upload-invalid: select a regular file",
        ));
    }
    let canonical = source.canonicalize().map_err(|error| error.to_string())?;
    if !app.fs_scope().is_allowed(&canonical) {
        return Err(AppError::new(
            "assistant_upload_unauthorized",
            "assistant-upload-unauthorized: resolved path is outside the selected scope",
        ));
    }
    Ok(canonical)
}

pub(super) fn classify(path: &Path) -> Result<UploadKind, AppError> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match extension.as_str() {
        "png" => Ok(UploadKind::Image("image/png")),
        "jpg" | "jpeg" => Ok(UploadKind::Image("image/jpeg")),
        "webp" => Ok(UploadKind::Image("image/webp")),
        "txt" | "md" | "markdown" | "csv" | "json" | "log" => Ok(UploadKind::Text),
        "pdf" => Ok(UploadKind::Pdf),
        "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" => Ok(UploadKind::Office),
        _ => Err(AppError::new(
            "assistant_upload_unsupported",
            "assistant-upload-unsupported: file type is not supported",
        )),
    }
}

pub(super) fn enforce_size(kind: UploadKind, size: u64) -> Result<(), AppError> {
    if size == 0
        || size > MAX_FILE_BYTES
        || matches!(kind, UploadKind::Image(_)) && size > MAX_IMAGE_BYTES
    {
        return Err(AppError::new(
            "assistant_upload_size",
            "assistant-upload-size: file is empty or exceeds its size limit",
        ));
    }
    Ok(())
}

pub(super) fn read_image(path: &Path, media_type: &str) -> Result<Vec<u8>, AppError> {
    let bytes = read_bounded(path, MAX_IMAGE_BYTES)?;
    let format = image::guess_format(&bytes).map_err(|error| {
        AppError::new(
            "assistant_upload_invalid",
            format!("assistant-upload-invalid: image format: {error}"),
        )
    })?;
    if !matches!(
        (media_type, format),
        ("image/png", image::ImageFormat::Png)
            | ("image/jpeg", image::ImageFormat::Jpeg)
            | ("image/webp", image::ImageFormat::WebP)
    ) {
        return Err(AppError::new(
            "assistant_upload_invalid",
            "assistant-upload-invalid: image content does not match its extension",
        ));
    }
    let (width, height) = image::ImageReader::with_format(std::io::Cursor::new(&bytes), format)
        .into_dimensions()
        .map_err(|error| {
            AppError::new(
                "assistant_upload_invalid",
                format!("assistant-upload-invalid: image dimensions: {error}"),
            )
        })?;
    if u64::from(width) * u64::from(height) > 24_000_000 {
        return Err(AppError::new(
            "assistant_upload_size",
            "assistant-upload-size: image exceeds 24 megapixels",
        ));
    }
    Ok(bytes)
}

pub(super) fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, AppError> {
    use std::io::Read;

    let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > limit {
        return Err(AppError::new(
            "assistant_upload_size",
            "assistant-upload-size: file exceeds its size limit",
        ));
    }
    Ok(bytes)
}

#[cfg(test)]
#[path = "tests/upload.rs"]
mod tests;
