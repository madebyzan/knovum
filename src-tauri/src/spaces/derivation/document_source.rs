use serde_json::Value;
use tauri::AppHandle;

use crate::documents::generation::{
    document::DocumentSource,
    input::{DocumentFileSource, DocumentInput},
};
use crate::foundation::app_error::AppError;

pub(super) async fn load(app: &AppHandle, request: &Value) -> Result<DocumentInput, AppError> {
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
        "markdown" => {
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
            Ok(DocumentInput::Markdown(DocumentSource {
                title,
                markdown: super::action_execution::required_text(document, "text")?.to_owned(),
                citations,
            }))
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
            Ok(DocumentInput::File(DocumentFileSource {
                title,
                path,
                source_revision: super::action_execution::required_text(
                    document,
                    "sourceRevision",
                )?
                .to_owned(),
            }))
        }
        _ => Err("action-input-invalid: unsupported document source".into()),
    }
}
