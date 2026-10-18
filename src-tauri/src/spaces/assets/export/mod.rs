mod document;
mod files;
mod image;
mod jobs;
mod media;
mod targets;

use std::path::{Path, PathBuf};

use serde_json::Value;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use self::targets::{Export, Managed};
use crate::foundation::app_error::AppError;
use crate::spaces::db;

/// What one resource can be saved as, and why anything further is unavailable.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFormats {
    /// The offered formats, in order; the first is the resource's own format.
    pub formats: Vec<String>,
    /// Why the engine offers nothing, when the resource's content is outside a contract. A
    /// resource whose kind simply has no engine format states nothing, because nothing is wrong.
    pub unavailable: Option<String>,
}

/// Lists the formats one resource can be saved as, own format first, and why
/// anything further is unavailable.
///
/// A conversion carries the same content as its source, so it belongs to the
/// destination the user picks rather than to a new resource on the canvas. The
/// chosen format is therefore the whole format instruction, and this is the only
/// place that decides which formats a resource admits.
#[tauri::command]
pub async fn list_export_formats(
    app: AppHandle,
    space_id: String,
    node_id: String,
) -> Result<ExportFormats, AppError> {
    Ok(targets::of(&load(&app, &space_id, &node_id).await?)?)
}

#[tauri::command]
pub async fn export_resource(
    app: AppHandle,
    space_id: String,
    node_id: String,
    destination_path: String,
    job_id: String,
) -> Result<(), AppError> {
    let (cancellation, _lease) = jobs::begin(&job_id)?;
    let payload = load(&app, &space_id, &node_id).await?;
    let written = write(
        app,
        &node_id,
        payload,
        PathBuf::from(destination_path),
        &cancellation,
    )
    .await;
    if cancellation.is_cancelled() {
        return Err(AppError::cancelled("The export was cancelled"));
    }
    written
}

async fn write(
    app: AppHandle,
    node_id: &str,
    payload: Value,
    destination: PathBuf,
    cancellation: &CancellationToken,
) -> Result<(), AppError> {
    match targets::requested(&payload, &destination)? {
        Export::Document(format) => {
            document::write(&app, node_id, &payload, format, &destination, cancellation).await
        }
        Export::Managed(managed) => {
            let source = managed_source(&payload)?;
            let cancellation = cancellation.clone();
            Ok(crate::foundation::blocking::run(move || match managed {
                Managed::Original => files::copy(&app, &source.path, &destination),
                Managed::Image(encoding) => {
                    image::write(&app, &source, &payload, encoding, &destination)
                }
                Managed::Media(target) => media::transcode(
                    &app,
                    &source.path,
                    &source.revision,
                    target,
                    &destination,
                    &cancellation,
                ),
            })
            .await?)
        }
    }
}

#[tauri::command]
pub fn cancel_resource_export(job_id: String) -> Result<bool, AppError> {
    Ok(jobs::cancel(&job_id)?)
}

/// Saves a managed file the Renderer already knows the path of, such as a
/// published speech stem, without reading it as a canvas resource.
#[tauri::command]
pub async fn export_asset(
    app: AppHandle,
    source_path: String,
    destination_path: String,
) -> Result<(), AppError> {
    Ok(crate::foundation::blocking::run(move || {
        files::copy(&app, Path::new(&source_path), Path::new(&destination_path))
    })
    .await?)
}

/// Saves text the Renderer serialized from content it already holds, such as a
/// note, a website summary or a section outline.
#[tauri::command]
pub async fn export_text(
    app: AppHandle,
    destination_path: String,
    text: String,
) -> Result<(), AppError> {
    Ok(crate::foundation::blocking::run(move || {
        files::write(&app, Path::new(&destination_path), text.as_bytes())
    })
    .await?)
}

/// A managed asset, as the database records it.
struct ManagedSource {
    path: PathBuf,
    revision: String,
}

fn managed_source(payload: &Value) -> Result<ManagedSource, String> {
    Ok(ManagedSource {
        path: PathBuf::from(
            payload
                .get("assetPath")
                .and_then(Value::as_str)
                .ok_or("The resource file is unavailable")?,
        ),
        revision: payload
            .get("sourceRevision")
            .and_then(Value::as_str)
            .ok_or("The resource has no published revision")?
            .to_owned(),
    })
}

async fn load(app: &AppHandle, space_id: &str, node_id: &str) -> Result<Value, AppError> {
    let encoded: String = sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
    )
    .bind(node_id)
    .bind(space_id)
    .fetch_optional(db::pool(app).await?)
    .await
    .map_err(db::sql_error)?
    .ok_or("The resource is no longer in this Space")?;
    Ok(serde_json::from_str(&encoded)
        .map_err(db::json_error)
        .map_err(AppError::from)?)
}

#[cfg(test)]
mod tests;
