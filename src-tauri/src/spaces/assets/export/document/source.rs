use serde_json::Value;
use tauri::AppHandle;

use crate::documents::generation::{
    document::{DocumentSource, NoteSource},
    input::{DocumentFileSource, DocumentInput, PdfSource},
    DocumentFormat,
};
use crate::foundation::app_error::AppError;

/// Reads one resource as the document engine expects it: authorized files and
/// revision-pinned images.
pub(super) async fn load(
    app: &AppHandle,
    node_id: &str,
    payload: &Value,
    format: DocumentFormat,
) -> Result<DocumentInput, AppError> {
    let kind = required(payload, "kind")?;
    let title = title(kind, payload)?;
    match kind {
        // A Note and a text block are the same editable content, presented differently.
        "markdown" | "note" | "text" => editable(app, node_id, payload, format, title).await,
        "website" => Ok(authored(title, super::super::capture::summary(payload)?)),
        "code" => listing(app, payload, title).await,
        "file" => file(app, payload, title),
        _ => Err(format!("A {kind} resource is not a document source").into()),
    }
}

/// A document the engine assembled from what the Space recorded, which points at no files of its
/// own: a capture's summary carries its pictures as addresses nobody imported.
fn authored(title: String, markdown: String) -> DocumentInput {
    DocumentInput::Markdown(DocumentSource {
        title,
        markdown,
        citations: vec![],
        images: vec![],
    })
}

/// A code resource is set as the listing it is: the characters of the file, fenced so that
/// nothing inside it can become markup.
async fn listing(
    app: &AppHandle,
    payload: &Value,
    title: String,
) -> Result<DocumentInput, AppError> {
    let file = managed_file(app, payload, String::new())?;
    let language = language(payload);
    let text = crate::foundation::blocking::run(move || {
        let bytes = crate::documents::generation::input::file::read_snapshot(
            &file,
            crate::documents::generation::MAX_AUTHORING_BYTES as u64,
        )?;
        String::from_utf8(bytes).map_err(|_| "This resource is not text".to_owned())
    })
    .await?;
    Ok(authored(title, fenced(&text, &language)))
}

/// The fence is longer than any run of backticks in the file, so the listing cannot end itself.
fn fenced(text: &str, language: &str) -> String {
    let mut longest = 0usize;
    let mut run = 0usize;
    for character in text.chars() {
        run = if character == '`' { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    let fence = "`".repeat(longest.max(2) + 1);
    format!("{fence}{language}\n{text}\n{fence}\n")
}

/// The language the Space recognized, as an info string: a word, or nothing.
fn language(payload: &Value) -> String {
    payload
        .get("previewLanguage")
        .and_then(Value::as_str)
        .filter(|value| {
            !value.is_empty()
                && value.chars().all(|character| {
                    character.is_ascii_alphanumeric() || character == '+' || character == '-'
                })
        })
        .unwrap_or_default()
        .to_ascii_lowercase()
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
    let file = managed_file(app, payload, title)?;
    if file
        .path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
    {
        return Ok(DocumentInput::Pdf(PdfSource { file }));
    }
    Ok(DocumentInput::File(file))
}

/// The managed file behind a resource, authorized and pinned to the revision it was published at.
fn managed_file(
    app: &AppHandle,
    payload: &Value,
    title: String,
) -> Result<DocumentFileSource, AppError> {
    let path = required(payload, "assetPath")?;
    if std::fs::symlink_metadata(path)
        .map_err(|error| error.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err("Document source must not be a symbolic link".into());
    }
    Ok(DocumentFileSource {
        title,
        path: crate::spaces::assets::authorize_spaces_asset(app, std::path::Path::new(path))?,
        source_revision: required(payload, "sourceRevision")?.to_owned(),
    })
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
