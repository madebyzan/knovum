use sqlx::{Row, Sqlite, SqlitePool, Transaction};

use crate::spaces::{conversations::Conversation, db};

pub async fn ensure_conversation(
    tx: &mut Transaction<'_, Sqlite>,
    conversation_id: &str,
    space_id: &str,
    title: &str,
    now: i64,
) -> Result<(), String> {
    let conversation_write = sqlx::query(
        "INSERT INTO spaces_conversations(id,space_id,title,active_branch_id,created_at,updated_at)
         VALUES($1,$2,$3,$1,$4,$4) ON CONFLICT(id) DO UPDATE SET updated_at=excluded.updated_at
         WHERE spaces_conversations.space_id=excluded.space_id",
    )
    .bind(conversation_id)
    .bind(space_id)
    .bind(title)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(conversation_write, "conversation-unavailable")
}

pub async fn ensure_branch_idle(
    tx: &mut Transaction<'_, Sqlite>,
    conversation_id: &str,
    branch_id: &str,
) -> Result<(), String> {
    let active: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_runs r WHERE r.completed_at IS NULL AND (
           EXISTS (SELECT 1 FROM spaces_builtin_agent_tasks t WHERE t.run_id=r.id
             AND t.conversation_id=$1 AND t.branch_id=$2)
           OR EXISTS (SELECT 1 FROM spaces_external_agent_conversations e WHERE e.run_id=r.id
             AND e.conversation_id=$1 AND e.branch_id=$2))",
    )
    .bind(conversation_id)
    .bind(branch_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if active != 0 {
        return Err(
            "conversation-branch-busy: finish or cancel the active task before sending a follow-up"
                .into(),
        );
    }
    Ok(())
}

pub async fn list_conversations(
    pool: &SqlitePool,
    space_id: &str,
    query: Option<&str>,
    archived: bool,
) -> Result<Vec<Conversation>, String> {
    let query = query.map(str::trim).filter(|value| !value.is_empty());
    let pattern = query.map(|value| format!("%{value}%"));
    let rows = sqlx::query("SELECT c.id,c.space_id,c.title,c.updated_at,c.active_branch_id AS branch_id
         FROM spaces_conversations c
         WHERE space_id=$1 AND (($2=1 AND archived_at IS NOT NULL) OR ($2=0 AND archived_at IS NULL))
         AND ($3 IS NULL OR c.title LIKE $3 ESCAPE '\\' OR EXISTS (
           SELECT 1 FROM spaces_conversation_messages m WHERE m.conversation_id=c.id AND m.content LIKE $3 ESCAPE '\\'
         )) ORDER BY updated_at DESC LIMIT 200")
        .bind(space_id).bind(archived).bind(pattern).fetch_all(pool).await.map_err(db::sql_error)?;
    rows.into_iter().map(parse).collect()
}

pub async fn rename(pool: &SqlitePool, conversation_id: &str, title: &str) -> Result<(), String> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 160 {
        return Err("conversation-title-invalid".into());
    }
    let conversation_write =
        sqlx::query("UPDATE spaces_conversations SET title=$1,updated_at=$2 WHERE id=$3")
            .bind(title)
            .bind(crate::foundation::clock::unix_time_millis()?)
            .bind(conversation_id)
            .execute(pool)
            .await
            .map_err(db::sql_error)?;
    db::require_one(conversation_write, "conversation-unavailable")
}

pub async fn archive(pool: &SqlitePool, conversation_id: &str) -> Result<(), String> {
    set_archived(pool, conversation_id, true).await
}

pub async fn unarchive(pool: &SqlitePool, conversation_id: &str) -> Result<(), String> {
    set_archived(pool, conversation_id, false).await
}

async fn set_archived(
    pool: &SqlitePool,
    conversation_id: &str,
    archived: bool,
) -> Result<(), String> {
    let now = crate::foundation::clock::unix_time_millis()?;
    let archived_at = archived.then_some(now);
    let conversation_write =
        sqlx::query("UPDATE spaces_conversations SET archived_at=$1,updated_at=$2 WHERE id=$3")
            .bind(archived_at)
            .bind(now)
            .bind(conversation_id)
            .execute(pool)
            .await
            .map_err(db::sql_error)?;
    db::require_one(conversation_write, "conversation-unavailable")
}

pub async fn delete(pool: &SqlitePool, conversation_id: &str) -> Result<(), String> {
    let active: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_builtin_agent_tasks t JOIN spaces_runs r ON r.id=t.run_id \
         WHERE t.conversation_id=$1 AND r.completed_at IS NULL",
    )
    .bind(conversation_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    if active != 0 {
        return Err("conversation-busy".into());
    }
    let mut tx = db::begin_write(pool).await?;
    sqlx::query(
        "DELETE FROM spaces_action_invocations WHERE principal_kind='builtin_agent' AND principal_id IN
         (SELECT id FROM spaces_builtin_agent_tasks WHERE conversation_id=$1)",
    )
    .bind(conversation_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "WITH RECURSIVE run_tree(id) AS (
           SELECT run_id FROM spaces_builtin_agent_tasks WHERE conversation_id=$1
           UNION ALL SELECT child.id FROM spaces_runs child JOIN run_tree parent
             ON child.parent_run_id=parent.id
         ) DELETE FROM spaces_runs WHERE id IN (SELECT id FROM run_tree)",
    )
    .bind(conversation_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query("DELETE FROM spaces_conversation_drafts WHERE conversation_id=$1")
        .bind(conversation_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    let conversation_write = sqlx::query("DELETE FROM spaces_conversations WHERE id=$1")
        .bind(conversation_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    db::require_one(conversation_write, "conversation-unavailable")?;
    sqlx::query(
        "DELETE FROM spaces_authorization_grants WHERE principal_kind='builtin_agent'
         AND NOT EXISTS (SELECT 1 FROM spaces_builtin_agent_tasks t WHERE t.grant_id=spaces_authorization_grants.id)",
    ).execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)
}

fn parse(row: sqlx::sqlite::SqliteRow) -> Result<Conversation, String> {
    Ok(Conversation {
        id: row.try_get("id").map_err(db::sql_error)?,
        space_id: row.try_get("space_id").map_err(db::sql_error)?,
        branch_id: row.try_get("branch_id").map_err(db::sql_error)?,
        title: row.try_get("title").map_err(db::sql_error)?,
        updated_at: row.try_get("updated_at").map_err(db::sql_error)?,
    })
}
