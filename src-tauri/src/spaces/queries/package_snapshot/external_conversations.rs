use super::decode::{decode_optional_json, decode_text};
use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

pub(super) async fn load(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<(Vec<Value>, Vec<Value>), String> {
    let conversations = sqlx::query(
        "SELECT id,run_id,assistant_conversation_id,branch_id,runtime_kind,runtime_session_id,session_capabilities,title,state,error,created_at,updated_at
         FROM spaces_external_agent_conversations WHERE space_id=$1 ORDER BY created_at,id",
    ).bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    let turns = sqlx::query(
        "SELECT t.* FROM spaces_external_agent_turns t JOIN spaces_external_agent_conversations c
         ON c.id=t.conversation_id WHERE c.space_id=$1 ORDER BY t.created_at,t.id",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok((
        conversations
            .into_iter()
            .map(decode_conversation_projection)
            .collect::<Result<_, _>>()?,
        turns
            .into_iter()
            .map(decode_turn_projection)
            .collect::<Result<_, _>>()?,
    ))
}

fn decode_conversation_projection(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    Ok(serde_json::json!({
        "id": decode_text(&row, "id")?, "runId": decode_text(&row, "run_id")?,
        "assistantConversationId": decode_text(&row, "assistant_conversation_id")?,
        "branchId": decode_text(&row, "branch_id")?,
        "runtimeKind": decode_text(&row, "runtime_kind")?,
        "runtimeSessionId": row.try_get::<Option<String>, _>("runtime_session_id").map_err(db::sql_error)?,
        "sessionCapabilities": decode_optional_json(&row, "session_capabilities")?,
        "title": decode_text(&row, "title")?, "state": decode_text(&row, "state")?,
        "error": row.try_get::<Option<String>, _>("error").map_err(db::sql_error)?,
        "createdAt": row.try_get::<i64, _>("created_at").map_err(db::sql_error)?,
        "updatedAt": row.try_get::<i64, _>("updated_at").map_err(db::sql_error)?,
    }))
}

fn decode_turn_projection(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    Ok(serde_json::json!({
        "id": decode_text(&row, "id")?, "conversationId": decode_text(&row, "conversation_id")?,
        "assistantMessageId": row.try_get::<Option<String>, _>("assistant_message_id").map_err(db::sql_error)?,
        "state": decode_text(&row, "state")?,
        "error": row.try_get::<Option<String>, _>("error").map_err(db::sql_error)?,
        "createdAt": row.try_get::<i64, _>("created_at").map_err(db::sql_error)?,
        "completedAt": row.try_get::<Option<i64>, _>("completed_at").map_err(db::sql_error)?,
    }))
}
