use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use ts_rs::TS;

use crate::spaces::db;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "AssistantMessage.ts")]
#[serde(rename_all = "camelCase")]
pub struct AssistantMessage {
    pub id: String,
    pub conversation_id: String,
    pub branch_id: String,
    pub role: String,
    pub content: AssistantMessageContent,
    pub created_at: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssistantMessageContent {
    pub parts: Vec<crate::ai::contracts::ModelContentPart>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<AssistantMessageAttachment>,
    #[serde(default)]
    #[ts(type = "Array<unknown>")]
    pub citations: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<AssistantMessageStatus>,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export_to = "AssistantMessageAttachment.ts")]
#[serde(
    tag = "source",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AssistantMessageAttachment {
    Node { title: String, node_id: String },
    File { title: String },
}

pub(crate) fn attachment_summaries(
    manifest: &Value,
) -> Result<Vec<AssistantMessageAttachment>, String> {
    let items = manifest
        .as_array()
        .ok_or("assistant-context-manifest-invalid")?;
    items
        .iter()
        .map(|item| match item.get("source").and_then(Value::as_str) {
            Some("node") => Ok(AssistantMessageAttachment::Node {
                title: required_attachment_field(item, "title")?,
                node_id: required_attachment_field(item, "nodeId")?,
            }),
            Some("file") => Ok(AssistantMessageAttachment::File {
                title: required_attachment_field(item, "name")?,
            }),
            _ => Err("assistant-context-manifest-invalid: attachment source".into()),
        })
        .collect()
}

fn required_attachment_field(item: &Value, key: &str) -> Result<String, String> {
    item.get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("assistant-context-manifest-invalid: {key}"))
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AssistantMessageStatus {
    Stopped,
    Incomplete,
}

pub async fn append_message(
    pool: &SqlitePool,
    conversation_id: &str,
    branch_id: &str,
    role: &str,
    content: &Value,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO spaces_assistant_messages(id,conversation_id,branch_id,role,content,created_at)
         VALUES($1,$2,$3,$4,$5,$6)")
        .bind(&id).bind(conversation_id).bind(branch_id).bind(role)
        .bind(serde_json::to_string(content).map_err(db::json_error)?)
        .bind(crate::clock::unix_time_millis()?).execute(pool).await.map_err(db::sql_error)?;
    Ok(id)
}

pub async fn append_message_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    conversation_id: &str,
    branch_id: &str,
    role: &str,
    content: &Value,
    now: i64,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO spaces_assistant_messages(id,conversation_id,branch_id,role,content,created_at)
         VALUES($1,$2,$3,$4,$5,$6)")
        .bind(&id).bind(conversation_id).bind(branch_id).bind(role)
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
) -> Result<Vec<AssistantMessage>, String> {
    let rows = sqlx::query(
        "SELECT id,conversation_id,branch_id,role,content,created_at
        FROM spaces_assistant_messages WHERE conversation_id=$1 AND branch_id=$2
        AND ($3 IS NULL OR (created_at,rowid) <
             ($3,(SELECT rowid FROM spaces_assistant_messages
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
) -> Result<Vec<AssistantMessage>, String> {
    let rows = sqlx::query(
        "SELECT id,conversation_id,branch_id,role,content,created_at
        FROM spaces_assistant_messages WHERE conversation_id=$1 AND branch_id=$2
        ORDER BY created_at,rowid",
    )
    .bind(conversation_id)
    .bind(branch_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter().map(parse).collect()
}

pub async fn branch(
    pool: &SqlitePool,
    conversation_id: &str,
    source_branch_id: &str,
    through_message_id: &str,
    include_boundary: bool,
) -> Result<String, String> {
    let boundary: (i64, i64) = sqlx::query_as(
        "SELECT created_at,rowid FROM spaces_assistant_messages WHERE id=$1 AND conversation_id=$2
         AND branch_id=$3",
    )
    .bind(through_message_id)
    .bind(conversation_id)
    .bind(source_branch_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("assistant-message-unavailable: branch point is unavailable")?;
    let comparison = if include_boundary { "<=" } else { "<" };
    let query = format!(
        "SELECT role,content,created_at FROM spaces_assistant_messages
         WHERE conversation_id=$1 AND branch_id=$2 AND (created_at,rowid) {comparison} ($3,$4)
         ORDER BY created_at,rowid",
    );
    let rows = sqlx::query(&query)
        .bind(conversation_id)
        .bind(source_branch_id)
        .bind(boundary.0)
        .bind(boundary.1)
        .fetch_all(pool)
        .await
        .map_err(db::sql_error)?;
    let branch_id = uuid::Uuid::new_v4().to_string();
    let now = crate::clock::unix_time_millis()?;
    let message_count = rows.len();
    let mut tx = db::begin_write(pool).await?;
    for (index, row) in rows.into_iter().enumerate() {
        sqlx::query(
            "INSERT INTO spaces_assistant_messages(id,conversation_id,branch_id,role,content,created_at)
             VALUES($1,$2,$3,$4,$5,$6)",
        ).bind(uuid::Uuid::new_v4().to_string()).bind(conversation_id).bind(&branch_id)
            .bind(row.try_get::<String, _>("role").map_err(db::sql_error)?)
            .bind(row.try_get::<String, _>("content").map_err(db::sql_error)?)
            .bind(now + i64::try_from(index).map_err(|_| "assistant-branch-limit")?)
            .execute(&mut *tx).await.map_err(db::sql_error)?;
    }
    sqlx::query(
        "UPDATE spaces_assistant_conversations SET updated_at=$1,active_branch_id=$2 WHERE id=$3",
    )
    .bind(now + i64::try_from(message_count).map_err(|_| "assistant-branch-limit")?)
    .bind(&branch_id)
    .bind(conversation_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(branch_id)
}

fn parse(row: sqlx::sqlite::SqliteRow) -> Result<AssistantMessage, String> {
    let content: String = row.try_get("content").map_err(db::sql_error)?;
    let content: AssistantMessageContent =
        serde_json::from_str(&content).map_err(db::json_error)?;
    crate::spaces::knowledge::grounding::validate_citation_snapshots(&Value::Array(
        content.citations.clone(),
    ))?;
    Ok(AssistantMessage {
        id: row.try_get("id").map_err(db::sql_error)?,
        conversation_id: row.try_get("conversation_id").map_err(db::sql_error)?,
        branch_id: row.try_get("branch_id").map_err(db::sql_error)?,
        role: row.try_get("role").map_err(db::sql_error)?,
        content,
        created_at: row.try_get("created_at").map_err(db::sql_error)?,
    })
}
