use serde_json::Value;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};

use crate::spaces::{
    assistant::{AssistantMode, ConversationSummary},
    db,
};

pub async fn ensure_conversation(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
    space_id: &str,
    title: &str,
    mode: AssistantMode,
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO spaces_assistant_conversations(id,space_id,title,mode,created_at,updated_at)
         VALUES($1,$2,$3,$4,$5,$5) ON CONFLICT(id) DO UPDATE SET updated_at=excluded.updated_at",
    )
    .bind(id)
    .bind(space_id)
    .bind(title)
    .bind(mode.as_str())
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub async fn list_conversations(
    pool: &SqlitePool,
    space_id: &str,
    query: Option<&str>,
    archived: bool,
) -> Result<Vec<ConversationSummary>, String> {
    let query = query.map(str::trim).filter(|value| !value.is_empty());
    let pattern = query.map(|value| format!("%{value}%"));
    let rows = sqlx::query("SELECT c.id,c.space_id,c.title,c.mode,c.updated_at,
         (SELECT m.branch_id FROM spaces_assistant_messages m WHERE m.conversation_id=c.id ORDER BY m.created_at DESC,m.id DESC LIMIT 1) branch_id
         FROM spaces_assistant_conversations c
         WHERE space_id=$1 AND (($2=1 AND archived_at IS NOT NULL) OR ($2=0 AND archived_at IS NULL))
         AND ($3 IS NULL OR c.title LIKE $3 ESCAPE '\\' OR EXISTS (
           SELECT 1 FROM spaces_assistant_messages m WHERE m.conversation_id=c.id AND m.content LIKE $3 ESCAPE '\\'
         )) ORDER BY updated_at DESC LIMIT 200")
        .bind(space_id).bind(archived).bind(pattern).fetch_all(pool).await.map_err(db::sql_error)?;
    rows.into_iter().map(parse).collect()
}

pub async fn rename(pool: &SqlitePool, id: &str, title: &str) -> Result<(), String> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 160 {
        return Err("assistant-conversation-title-invalid".into());
    }
    let result =
        sqlx::query("UPDATE spaces_assistant_conversations SET title=$1,updated_at=$2 WHERE id=$3")
            .bind(title)
            .bind(crate::clock::unix_time_millis()?)
            .bind(id)
            .execute(pool)
            .await
            .map_err(db::sql_error)?;
    db::require_one(result, "assistant-conversation-unavailable")
}

pub async fn archive(pool: &SqlitePool, id: &str, archived: bool) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let archived_at = archived.then_some(now);
    let result = sqlx::query(
        "UPDATE spaces_assistant_conversations SET archived_at=$1,updated_at=$2 WHERE id=$3",
    )
    .bind(archived_at)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    db::require_one(result, "assistant-conversation-unavailable")
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), String> {
    let active: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_assistant_tasks t JOIN spaces_runs r ON r.id=t.run_id \
         WHERE t.conversation_id=$1 AND r.completed_at IS NULL",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    if active != 0 {
        return Err("assistant-conversation-busy".into());
    }
    let mut tx = db::begin_write(pool).await?;
    sqlx::query(
        "DELETE FROM spaces_action_invocations WHERE principal_kind='assistant' AND principal_id IN
         (SELECT id FROM spaces_assistant_tasks WHERE conversation_id=$1)",
    )
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "WITH RECURSIVE run_tree(id) AS (
           SELECT run_id FROM spaces_assistant_tasks WHERE conversation_id=$1
           UNION ALL SELECT child.id FROM spaces_runs child JOIN run_tree parent
             ON child.parent_run_id=parent.id
         ) DELETE FROM spaces_runs WHERE id IN (SELECT id FROM run_tree)",
    )
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let result = sqlx::query("DELETE FROM spaces_assistant_conversations WHERE id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    db::require_one(result, "assistant-conversation-unavailable")?;
    sqlx::query(
        "DELETE FROM spaces_authorization_grants WHERE principal_kind='assistant'
         AND NOT EXISTS (SELECT 1 FROM spaces_assistant_tasks t WHERE t.grant_id=spaces_authorization_grants.id)",
    ).execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)
}

fn parse(row: sqlx::sqlite::SqliteRow) -> Result<ConversationSummary, String> {
    let mode: String = row.try_get("mode").map_err(db::sql_error)?;
    Ok(ConversationSummary {
        id: row.try_get("id").map_err(db::sql_error)?,
        space_id: row.try_get("space_id").map_err(db::sql_error)?,
        branch_id: row.try_get("branch_id").map_err(db::sql_error)?,
        title: row.try_get("title").map_err(db::sql_error)?,
        mode: serde_json::from_value(Value::String(mode)).map_err(db::json_error)?,
        updated_at: row.try_get("updated_at").map_err(db::sql_error)?,
    })
}
