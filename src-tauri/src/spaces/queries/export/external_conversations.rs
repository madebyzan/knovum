use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

pub(super) async fn rows(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<(Vec<Value>, Vec<Value>, Vec<Value>), String> {
    let conversations = sqlx::query(
        "SELECT id,run_id,runtime_kind,runtime_session_id,session_capabilities,title,state,error,created_at,updated_at
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
    let messages = sqlx::query(
        "SELECT m.* FROM spaces_external_agent_messages m JOIN spaces_external_agent_conversations c
         ON c.id=m.conversation_id WHERE c.space_id=$1 ORDER BY m.created_at,m.id",
    ).bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    Ok((
        conversations
            .into_iter()
            .map(conversation)
            .collect::<Result<_, _>>()?,
        turns.into_iter().map(turn).collect::<Result<_, _>>()?,
        messages
            .into_iter()
            .map(message)
            .collect::<Result<_, _>>()?,
    ))
}

fn conversation(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    Ok(serde_json::json!({
        "id": text(&row, "id")?, "runId": text(&row, "run_id")?,
        "runtimeKind": text(&row, "runtime_kind")?,
        "runtimeSessionId": row.try_get::<Option<String>, _>("runtime_session_id").map_err(db::sql_error)?,
        "sessionCapabilities": optional_json(&row, "session_capabilities")?,
        "title": text(&row, "title")?, "state": text(&row, "state")?,
        "error": row.try_get::<Option<String>, _>("error").map_err(db::sql_error)?,
        "createdAt": row.try_get::<i64, _>("created_at").map_err(db::sql_error)?,
        "updatedAt": row.try_get::<i64, _>("updated_at").map_err(db::sql_error)?,
    }))
}

fn optional_json(row: &sqlx::sqlite::SqliteRow, field: &str) -> Result<Option<Value>, String> {
    row.try_get::<Option<String>, _>(field)
        .map_err(db::sql_error)?
        .map(|value| serde_json::from_str(&value).map_err(db::json_error))
        .transpose()
}

fn turn(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    Ok(serde_json::json!({
        "id": text(&row, "id")?, "conversationId": text(&row, "conversation_id")?,
        "state": text(&row, "state")?,
        "error": row.try_get::<Option<String>, _>("error").map_err(db::sql_error)?,
        "createdAt": row.try_get::<i64, _>("created_at").map_err(db::sql_error)?,
        "completedAt": row.try_get::<Option<i64>, _>("completed_at").map_err(db::sql_error)?,
    }))
}

fn message(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    Ok(serde_json::json!({
        "id": text(&row, "id")?, "conversationId": text(&row, "conversation_id")?,
        "turnId": text(&row, "turn_id")?, "role": text(&row, "role")?,
        "content": text(&row, "content")?,
        "createdAt": row.try_get::<i64, _>("created_at").map_err(db::sql_error)?,
    }))
}

fn text(row: &sqlx::sqlite::SqliteRow, field: &str) -> Result<String, String> {
    row.try_get(field).map_err(db::sql_error)
}
