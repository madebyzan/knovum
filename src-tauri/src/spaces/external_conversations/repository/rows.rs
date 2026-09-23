use sqlx::Row;

use super::{ExternalConversationMessage, ExternalConversationSummary};
use crate::spaces::db;

pub(super) fn conversation(
    row: sqlx::sqlite::SqliteRow,
) -> Result<ExternalConversationSummary, String> {
    let runtime: String = row.try_get("runtime_kind").map_err(db::sql_error)?;
    let capabilities: Option<String> =
        row.try_get("session_capabilities").map_err(db::sql_error)?;
    Ok(ExternalConversationSummary {
        id: row.try_get("id").map_err(db::sql_error)?,
        run_id: row.try_get("run_id").map_err(db::sql_error)?,
        space_id: row.try_get("space_id").map_err(db::sql_error)?,
        runtime_kind: serde_json::from_value(serde_json::Value::String(runtime))
            .map_err(db::json_error)?,
        runtime_session_id: row.try_get("runtime_session_id").map_err(db::sql_error)?,
        session_capabilities: capabilities
            .map(|value| serde_json::from_str(&value))
            .transpose()
            .map_err(db::json_error)?,
        title: row.try_get("title").map_err(db::sql_error)?,
        state: row.try_get("state").map_err(db::sql_error)?,
        error: row.try_get("error").map_err(db::sql_error)?,
        updated_at: row.try_get("updated_at").map_err(db::sql_error)?,
    })
}

pub(super) fn message(row: sqlx::sqlite::SqliteRow) -> Result<ExternalConversationMessage, String> {
    Ok(ExternalConversationMessage {
        id: row.try_get("id").map_err(db::sql_error)?,
        conversation_id: row.try_get("conversation_id").map_err(db::sql_error)?,
        turn_id: row.try_get("turn_id").map_err(db::sql_error)?,
        role: row.try_get("role").map_err(db::sql_error)?,
        content: row.try_get("content").map_err(db::sql_error)?,
        created_at: row.try_get("created_at").map_err(db::sql_error)?,
    })
}
