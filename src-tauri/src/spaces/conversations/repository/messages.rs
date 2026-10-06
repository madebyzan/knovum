use serde_json::Value;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};

use super::message_contract::{
    ConversationMessage, ConversationMessageContent, ConversationMessageRole,
};
use crate::spaces::db;

pub async fn append_message_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    conversation_id: &str,
    branch_id: &str,
    role: ConversationMessageRole,
    content: &Value,
    now: i64,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO spaces_conversation_messages(id,conversation_id,branch_id,role,content,created_at)
         VALUES($1,$2,$3,$4,$5,$6)")
        .bind(&id).bind(conversation_id).bind(branch_id).bind(role.as_str())
        .bind(serde_json::to_string(content).map_err(db::json_error)?)
        .bind(now).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(id)
}

pub async fn list_messages(
    pool: &SqlitePool,
    conversation_id: &str,
    branch_id: &str,
    before: Option<(i64, &str)>,
    limit: u16,
) -> Result<Vec<ConversationMessage>, String> {
    let rows = sqlx::query(
        "SELECT id,conversation_id,branch_id,role,content,created_at
        FROM spaces_conversation_messages WHERE conversation_id=$1 AND branch_id=$2
        AND ($3 IS NULL OR (created_at,rowid) <
             ($3,(SELECT rowid FROM spaces_conversation_messages
                  WHERE id=$4 AND conversation_id=$1 AND branch_id=$2)))
        ORDER BY created_at DESC,rowid DESC LIMIT $5",
    )
    .bind(conversation_id)
    .bind(branch_id)
    .bind(before.map(|cursor| cursor.0))
    .bind(before.map(|cursor| cursor.1))
    .bind(i64::from(limit.clamp(1, 200)))
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let mut messages = rows.into_iter().map(parse).collect::<Result<Vec<_>, _>>()?;
    messages.reverse();
    Ok(messages)
}

pub async fn all_messages(
    pool: &SqlitePool,
    conversation_id: &str,
    branch_id: &str,
) -> Result<Vec<ConversationMessage>, String> {
    let rows = sqlx::query(
        "SELECT id,conversation_id,branch_id,role,content,created_at
        FROM spaces_conversation_messages WHERE conversation_id=$1 AND branch_id=$2
        ORDER BY created_at,rowid",
    )
    .bind(conversation_id)
    .bind(branch_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter().map(parse).collect()
}

fn parse(row: sqlx::sqlite::SqliteRow) -> Result<ConversationMessage, String> {
    let content: String = row.try_get("content").map_err(db::sql_error)?;
    let content: ConversationMessageContent =
        serde_json::from_str(&content).map_err(db::json_error)?;
    crate::spaces::knowledge::grounding::validate_citation_snapshots(&Value::Array(
        content.citations.clone(),
    ))?;
    Ok(ConversationMessage {
        id: row.try_get("id").map_err(db::sql_error)?,
        conversation_id: row.try_get("conversation_id").map_err(db::sql_error)?,
        branch_id: row.try_get("branch_id").map_err(db::sql_error)?,
        role: row
            .try_get::<String, _>("role")
            .map_err(db::sql_error)?
            .parse()?,
        content,
        created_at: row.try_get("created_at").map_err(db::sql_error)?,
    })
}
