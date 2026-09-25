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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InspectedUpload {
    pub path: String,
    pub source_revision: String,
}

#[tauri::command]
pub async fn inspect_assistant_upload(
    app: tauri::AppHandle,
    path: String,
) -> Result<InspectedUpload, String> {
    crate::blocking::run(move || inspect(&app, &path)).await
}

fn inspect(app: &tauri::AppHandle, path: &str) -> Result<InspectedUpload, String> {
    let source = authorize(app, path)?;
    source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or("assistant-upload-invalid: filename is not Unicode")?;
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
            String::from_utf8(read_bounded(&source, MAX_FILE_BYTES)?)
                .map_err(|error| format!("assistant-upload-invalid: text is not UTF-8: {error}"))?;
        }
        UploadKind::Pdf | UploadKind::Office => {}
    }
    Ok(InspectedUpload {
        path: source.to_string_lossy().into_owned(),
        source_revision: crate::file_inspection::content_revision(&source)?,
    })
}

pub(super) fn authorize(app: &tauri::AppHandle, path: &str) -> Result<PathBuf, String> {
    let source = Path::new(path);
    crate::spaces::assets::authorize_source(app, source)?;
    if !source.is_file() {
        return Err("assistant-upload-invalid: select a regular file".into());
    }
    let canonical = source.canonicalize().map_err(|error| error.to_string())?;
    if !app.fs_scope().is_allowed(&canonical) {
        return Err(
            "assistant-upload-unauthorized: resolved path is outside the selected scope".into(),
        );
    }
    Ok(canonical)
}

pub(super) fn classify(path: &Path) -> Result<UploadKind, String> {
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
        _ => Err("assistant-upload-unsupported: file type is not supported".into()),
    }
}

pub(super) fn enforce_size(kind: UploadKind, size: u64) -> Result<(), String> {
    if size == 0
        || size > MAX_FILE_BYTES
        || matches!(kind, UploadKind::Image(_)) && size > MAX_IMAGE_BYTES
    {
        return Err("assistant-upload-size: file is empty or exceeds its size limit".into());
    }
    Ok(())
}

pub(super) fn read_image(path: &Path, media_type: &str) -> Result<Vec<u8>, String> {
    let bytes = read_bounded(path, MAX_IMAGE_BYTES)?;
    let format = image::guess_format(&bytes)
        .map_err(|error| format!("assistant-upload-invalid: image format: {error}"))?;
    if !matches!(
        (media_type, format),
        ("image/png", image::ImageFormat::Png)
            | ("image/jpeg", image::ImageFormat::Jpeg)
            | ("image/webp", image::ImageFormat::WebP)
    ) {
        return Err("assistant-upload-invalid: image content does not match its extension".into());
    }
    let (width, height) = image::ImageReader::with_format(std::io::Cursor::new(&bytes), format)
        .into_dimensions()
        .map_err(|error| format!("assistant-upload-invalid: image dimensions: {error}"))?;
    if u64::from(width) * u64::from(height) > 24_000_000 {
        return Err("assistant-upload-size: image exceeds 24 megapixels".into());
    }
    Ok(bytes)
}

pub(super) fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    use std::io::Read;

    let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("assistant-upload-size: file exceeds its size limit".into());
    }
    Ok(bytes)
}
