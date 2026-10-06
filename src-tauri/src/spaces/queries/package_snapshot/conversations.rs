use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

pub(super) async fn load(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<(Vec<Value>, Vec<Value>), String> {
    let conversations = sqlx::query(
        "SELECT id,title,active_branch_id,archived_at,created_at,updated_at
         FROM spaces_conversations WHERE space_id=$1 ORDER BY created_at,id",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let messages = sqlx::query(
        "SELECT m.id,m.conversation_id,m.branch_id,m.parent_message_id,m.role,m.content,m.created_at
         FROM spaces_conversation_messages m JOIN spaces_conversations c ON c.id=m.conversation_id
         WHERE c.space_id=$1 ORDER BY m.created_at,m.rowid",
    ).bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    Ok((
        conversations
            .into_iter()
            .map(decode_conversation_projection)
            .collect::<Result<_, _>>()?,
        messages
            .into_iter()
            .map(decode_message_projection)
            .collect::<Result<_, _>>()?,
    ))
}

fn decode_conversation_projection(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    Ok(serde_json::json!({
        "id": row.try_get::<String, _>("id").map_err(db::sql_error)?,
        "title": row.try_get::<String, _>("title").map_err(db::sql_error)?,
        "activeBranchId": row.try_get::<String, _>("active_branch_id").map_err(db::sql_error)?,
        "archivedAt": row.try_get::<Option<i64>, _>("archived_at").map_err(db::sql_error)?,
        "createdAt": row.try_get::<i64, _>("created_at").map_err(db::sql_error)?,
        "updatedAt": row.try_get::<i64, _>("updated_at").map_err(db::sql_error)?,
    }))
}

fn decode_message_projection(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    let content: String = row.try_get("content").map_err(db::sql_error)?;
    Ok(serde_json::json!({
        "id": row.try_get::<String, _>("id").map_err(db::sql_error)?,
        "conversationId": row.try_get::<String, _>("conversation_id").map_err(db::sql_error)?,
        "branchId": row.try_get::<String, _>("branch_id").map_err(db::sql_error)?,
        "parentMessageId": row.try_get::<Option<String>, _>("parent_message_id").map_err(db::sql_error)?,
        "role": row.try_get::<String, _>("role").map_err(db::sql_error)?,
        "content": serde_json::from_str::<Value>(&content).map_err(db::json_error)?,
        "createdAt": row.try_get::<i64, _>("created_at").map_err(db::sql_error)?,
    }))
}

#[cfg(test)]
#[path = "tests/conversations.rs"]
mod tests;
