use serde_json::Value;
use tauri::AppHandle;

use crate::documents::generation::{
    document::{DocumentSource, NoteSource},
    input::{DocumentFileSource, DocumentInput},
};
use crate::foundation::app_error::AppError;

pub(super) async fn load(
    app: &AppHandle,
    request: &Value,
    format: crate::documents::generation::DocumentFormat,
) -> Result<DocumentInput, AppError> {
    let document = request
        .pointer("/document/data")
        .and_then(Value::as_object)
        .ok_or("action-input-invalid: source document data is required")?;
    let kind = request
        .pointer("/document/kind")
        .and_then(Value::as_str)
        .ok_or("Document source kind is required")?;
    let title = super::document_formats::resolve_source_title(kind, &request["document"]["data"])?;
    match kind {
        "markdown" | "note" => {
            let node_id = request
                .pointer("/document/nodeId")
                .and_then(Value::as_str)
                .ok_or("action-input-invalid: source document identity is required")?;
            let mut tx = crate::spaces::db::pool(app)
                .await?
                .begin()
                .await
                .map_err(crate::spaces::db::sql_error)?;
            let citations = super::document_citations::load(&mut tx, node_id).await?;
            tx.commit().await.map_err(crate::spaces::db::sql_error)?;
            let markdown = if kind == "markdown" {
                Some(super::action_execution::required_text(document, "text")?.to_owned())
            } else {
                None
            };
            let images = if matches!(format, crate::documents::generation::DocumentFormat::Text) {
                vec![]
            } else {
                let image_app = app.clone();
                let content = request["document"]["data"].clone();
                let text = markdown.clone();
                crate::foundation::blocking::run(move || {
                    super::document_images::load(&image_app, &content, text.as_deref())
                })
                .await?
            };
            if let Some(markdown) = markdown {
                Ok(DocumentInput::Markdown(DocumentSource {
                    title,
                    markdown,
                    citations,
                    images,
                }))
            } else {
                Ok(DocumentInput::Note(NoteSource {
                    title,
                    content: document
                        .get("tiptapContent")
                        .ok_or("Note document content is required")?
                        .clone(),
                    citations,
                    images,
                }))
            }
        }
        "file" => {
            let path = super::action_execution::required_text(document, "assetPath")?;
            if std::fs::symlink_metadata(path)
                .map_err(|error| error.to_string())?
                .file_type()
                .is_symlink()
            {
                return Err("Document source must not be a symbolic link".into());
            }
            let path =
                crate::spaces::assets::authorize_spaces_asset(app, std::path::Path::new(path))?;
            let file = DocumentFileSource {
                title,
                path,
                source_revision: super::action_execution::required_text(
                    document,
                    "sourceRevision",
                )?
                .to_owned(),
            };
            if file
                .path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
            {
                Ok(DocumentInput::Pdf(
                    crate::documents::generation::input::PdfSource {
                        file,
                        recognized_pages: super::document_recognition::load(
                            &request["document"]["data"],
                        )?,
                    },
                ))
            } else {
                Ok(DocumentInput::File(file))
            }
        }
        _ => Err("action-input-invalid: unsupported document source".into()),
    }
}
