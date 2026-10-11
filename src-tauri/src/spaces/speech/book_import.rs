use std::{fs, io::Read, path::Path};

use serde::Serialize;
use tauri::AppHandle;

use super::cancellation;
use crate::{
    documents::extraction, file_inspection, foundation::app_error::AppError, spaces::assets,
};

const MAX_SOURCE_BYTES: u64 = 128 * 1024 * 1024;
const MAX_TEXT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct BookArchiveMember {
    pub path: String,
    pub text: String,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum AudiobookSource {
    Text { text: String },
    Epub { entries: Vec<BookArchiveMember> },
}

#[tauri::command]
pub async fn read_audiobook_source(
    app: AppHandle,
    job_id: String,
    source_node_id: String,
) -> Result<AudiobookSource, AppError> {
    let (cancellation, _lease) = cancellation::acquire(&job_id)?;
    let data: String = sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE id=? AND kind='file' AND deleted_at IS NULL",
    )
    .bind(&source_node_id)
    .fetch_optional(crate::spaces::db::pool(&app).await?)
    .await
    .map_err(crate::spaces::db::sql_error)?
    .ok_or("Audiobook document node is unavailable")?;
    let data: serde_json::Value =
        serde_json::from_str(&data).map_err(crate::spaces::db::json_error)?;
    let source_path = data["assetPath"]
        .as_str()
        .ok_or("Audiobook document has no managed asset")?;
    let expected_revision = data["sourceRevision"]
        .as_str()
        .ok_or("Audiobook document has no source revision")?
        .to_owned();
    let source = assets::authorize_spaces_asset(&app, Path::new(source_path))?;
    Ok(crate::foundation::blocking::run(move || -> Result<_, String> {
        let metadata = fs::metadata(&source).map_err(|error| error.to_string())?;
        if !metadata.is_file() || metadata.len() > MAX_SOURCE_BYTES { return Err("Audiobook source exceeds 128 MiB".into()); }
        file_inspection::verify_content_revision(&source, &expected_revision)?;
        let extension = source.extension().and_then(|extension| extension.to_str()).unwrap_or("").to_ascii_lowercase();
        let extraction = match extension.as_str() {
            "epub" => AudiobookSource::Epub { entries: read_epub(&source, &cancellation)? },
            "doc" | "docx" => {
                let document = extraction::office::extract(&source, &cancellation)?;
                AudiobookSource::Text { text: document.segments.into_iter().map(|segment| segment.text).collect::<Vec<_>>().join("\n\n") }
            }
            "pdf" => {
                let document = extraction::pdf::extract(&source, &cancellation)?;
                if !document.pages_requiring_recognition.is_empty() { return Err("Audiobook PDF has pages requiring recognition; resolve document recognition before importing its script".into()); }
                AudiobookSource::Text { text: document.segments.into_iter().map(|segment| segment.text).collect::<Vec<_>>().join("\n\n") }
            }
            "txt" | "md" | "json" => {
                if metadata.len() > MAX_TEXT_BYTES as u64 { return Err("Audiobook text source exceeds 16 MiB".into()); }
                AudiobookSource::Text { text: fs::read_to_string(&source).map_err(|error| error.to_string())? }
            }
            _ => return Err("Unsupported audiobook document format".into()),
        };
        if cancellation.is_cancelled() { return Err("cancelled".into()); }
        file_inspection::verify_content_revision(&source, &expected_revision)?;
        Ok(extraction)
    }).await?)
}

fn read_epub(
    source: &Path,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<Vec<BookArchiveMember>, String> {
    let file = fs::File::open(source).map_err(|error| error.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|error| error.to_string())?;
    if archive.len() > 10_000 {
        return Err("EPUB exceeds 10000 archive entries".into());
    }
    let mut entries = Vec::new();
    let mut total = 0_usize;
    for index in 0..archive.len() {
        if cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        let path = entry
            .enclosed_name()
            .ok_or("EPUB contains an unsafe archive path")?;
        if !matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("xml" | "opf" | "xhtml" | "html" | "htm")
        ) {
            continue;
        }
        if entry.size() > 4 * 1024 * 1024 {
            return Err("EPUB chapter exceeds 4 MiB".into());
        }
        let mut bytes = Vec::new();
        entry
            .by_ref()
            .take(4 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        total += bytes.len();
        if total > MAX_TEXT_BYTES || bytes.len() > 4 * 1024 * 1024 {
            return Err("EPUB extracted text exceeds 16 MiB".into());
        }
        entries.push(BookArchiveMember {
            path: path.to_string_lossy().replace('\\', "/"),
            text: String::from_utf8(bytes).map_err(|error| error.to_string())?,
        });
    }
    Ok(entries)
}
