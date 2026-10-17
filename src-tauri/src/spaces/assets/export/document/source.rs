use serde_json::Value;
use tauri::AppHandle;

use crate::documents::generation::{
    document::{DocumentSource, NoteSource},
    input::{DocumentFileSource, DocumentInput, PdfSource},
    DocumentFormat,
};
use crate::foundation::app_error::AppError;

/// Reads one resource as the document engine expects it: authorized files,
/// revision-pinned images and, for a scanned PDF, its qualified recognition.
pub(super) async fn load(
    app: &AppHandle,
    node_id: &str,
    payload: &Value,
    format: DocumentFormat,
) -> Result<DocumentInput, AppError> {
    let kind = required(payload, "kind")?;
    let title = title(kind, payload)?;
    match kind {
        "markdown" | "note" => editable(app, node_id, payload, format, title).await,
        "file" => file(app, payload, title),
        _ => Err(format!("A {kind} resource is not a document source").into()),
    }
}

async fn editable(
    app: &AppHandle,
    node_id: &str,
    payload: &Value,
    format: DocumentFormat,
    title: String,
) -> Result<DocumentInput, AppError> {
    let mut tx = crate::spaces::db::pool(app)
        .await?
        .begin()
        .await
        .map_err(crate::spaces::db::sql_error)?;
    let citations = super::citations::load(&mut tx, node_id).await?;
    tx.commit().await.map_err(crate::spaces::db::sql_error)?;
    let markdown = match required(payload, "kind")? {
        "markdown" => Some(required(payload, "text")?.to_owned()),
        _ => None,
    };
    // Plain text carries image alternative text, so it never needs the bytes.
    let images = if matches!(format, DocumentFormat::Text) {
        vec![]
    } else {
        let app = app.clone();
        let payload = payload.clone();
        let text = markdown.clone();
        crate::foundation::blocking::run(move || {
            super::images::load(&app, &payload, text.as_deref())
        })
        .await?
    };
    Ok(match markdown {
        Some(markdown) => DocumentInput::Markdown(DocumentSource {
            title,
            markdown,
            citations,
            images,
        }),
        None => DocumentInput::Note(NoteSource {
            title,
            content: payload
                .get("tiptapContent")
                .ok_or("Note document content is required")?
                .clone(),
            citations,
            images,
        }),
    })
}

fn file(app: &AppHandle, payload: &Value, title: String) -> Result<DocumentInput, AppError> {
    let path = required(payload, "assetPath")?;
    if std::fs::symlink_metadata(path)
        .map_err(|error| error.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err("Document source must not be a symbolic link".into());
    }
    let file = DocumentFileSource {
        title,
        path: crate::spaces::assets::authorize_spaces_asset(app, std::path::Path::new(path))?,
        source_revision: required(payload, "sourceRevision")?.to_owned(),
    };
    if file
        .path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
    {
        return Ok(DocumentInput::Pdf(PdfSource {
            recognized_pages: super::recognition::load(payload)?,
            file,
        }));
    }
    Ok(DocumentInput::File(file))
}

/// A file resource keeps its own name as its display title, which would read as
/// a heading inside the document, so the extension is dropped from it.
fn title(kind: &str, payload: &Value) -> Result<String, String> {
    let title = required(payload, "title")?;
    if kind == "file" && payload.get("fileName").and_then(Value::as_str) == Some(title) {
        return std::path::Path::new(title)
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .map(str::to_owned)
            .ok_or_else(|| "Document source filename is invalid".into());
    }
    Ok(title.into())
}

fn required<'a>(payload: &'a Value, field: &str) -> Result<&'a str, String> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Document source {field} is required"))
}
