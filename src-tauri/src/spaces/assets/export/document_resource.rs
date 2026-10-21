//! A picture a document carries, saved on its own.
//!
//! Parsing a document extracts every picture it shows as a durable PNG named by the digest of its
//! bytes, so a picture is addressed by its document and that digest, and its revision is the digest
//! itself. Saving one follows the same routes as a picture resource.

use std::path::PathBuf;

use serde_json::Value;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use super::{targets, ExportFormats, ManagedSource};
use crate::foundation::app_error::AppError;
use crate::spaces::knowledge::model::document_coverage::DocumentCoverage;

/// Lists the formats a document's picture can be saved as, its own format first.
#[tauri::command]
pub async fn list_document_resource_export_formats(
    app: AppHandle,
    space_id: String,
    node_id: String,
    resource_id: String,
) -> Result<ExportFormats, AppError> {
    locate(&super::load(&app, &space_id, &node_id).await?, &resource_id)?;
    Ok(ExportFormats {
        formats: targets::document_resource()
            .into_iter()
            .map(|(extension, _)| extension)
            .collect(),
        unavailable: None,
    })
}

/// Writes a document's picture into the destination, in the format the destination names.
#[tauri::command]
pub async fn export_document_resource(
    app: AppHandle,
    space_id: String,
    node_id: String,
    resource_id: String,
    destination_path: String,
) -> Result<(), AppError> {
    let source = locate(&super::load(&app, &space_id, &node_id).await?, &resource_id)?;
    let destination = PathBuf::from(destination_path);
    let managed = targets::chosen(targets::document_resource(), None, &destination)?;
    super::write_managed(
        app,
        source,
        managed,
        Value::Null,
        destination,
        CancellationToken::new(),
    )
    .await
}

/// The stored picture the document currently publishes under `resource_id`.
fn locate(payload: &Value, resource_id: &str) -> Result<ManagedSource, String> {
    let coverage = DocumentCoverage::from_node_content(payload)?;
    let resource = coverage
        .resources
        .into_iter()
        .find(|resource| resource.id == resource_id)
        .ok_or("The picture is no longer part of this document")?;
    Ok(ManagedSource {
        revision: format!("sha256:{}", resource.id),
        path: PathBuf::from(resource.path),
    })
}
