//! The text recognition read from a node's pictures, as each picture's own reading.

use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::{AppHandle, Runtime};

use crate::{foundation::app_error::AppError, spaces::db};

/// The metadata role of every passage recognition writes.
pub(crate) const RECOGNIZED_TEXT_ROLE: &str = "recognized_text";

/// What recognition read from one picture: a document's extracted picture by its resource, or the
/// picture an image or video shows when `resource_id` is absent.
#[derive(Debug, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct RecognizedTextReading {
    pub resource_id: Option<String>,
    pub text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RecognizedTextRequest {
    space_id: String,
    node_id: String,
}

#[tauri::command]
pub(crate) async fn list_recognized_text<R: Runtime>(
    app: AppHandle<R>,
    request: RecognizedTextRequest,
) -> Result<Vec<RecognizedTextReading>, AppError> {
    Ok(readings(db::pool(&app).await?, &request.space_id, &request.node_id).await?)
}

/// Passages are written in reading order, so insertion order rebuilds each picture's text.
pub(crate) async fn readings(
    pool: &sqlx::SqlitePool,
    space_id: &str,
    node_id: &str,
) -> Result<Vec<RecognizedTextReading>, String> {
    let rows = sqlx::query(
        "SELECT json_extract(c.metadata,'$.resourceId') AS resource_id, c.text \
         FROM spaces_knowledge_chunks c \
         JOIN spaces_nodes n ON n.id=c.node_id AND n.space_id=c.space_id \
         WHERE c.space_id=$1 AND c.node_id=$2 AND n.deleted_at IS NULL AND c.origin='source' \
           AND json_extract(c.metadata,'$.role')=$3 \
           AND c.source_revision=json_extract(n.data,'$.sourceRevision') \
         ORDER BY c.rowid",
    )
    .bind(space_id)
    .bind(node_id)
    .bind(RECOGNIZED_TEXT_ROLE)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let mut readings: Vec<RecognizedTextReading> = Vec::new();
    for row in rows {
        let resource_id: Option<String> = row.try_get("resource_id").map_err(db::sql_error)?;
        let text: String = row.try_get("text").map_err(db::sql_error)?;
        match readings.last_mut() {
            Some(reading) if reading.resource_id == resource_id => {
                reading.text.push('\n');
                reading.text.push_str(&text);
            }
            _ => readings.push(RecognizedTextReading { resource_id, text }),
        }
    }
    Ok(readings)
}

#[cfg(test)]
#[path = "tests/recognized_text.rs"]
mod tests;
