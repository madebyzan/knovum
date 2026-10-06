use sqlx::{Row, SqlitePool};

use crate::spaces::db;

pub async fn list(pool: &SqlitePool, conversation_id: &str) -> Result<Vec<String>, String> {
    sqlx::query_scalar(
        "SELECT branch_id FROM (
           SELECT branch_id,MIN(created_at) AS created_at FROM spaces_conversation_messages
           WHERE conversation_id=$1 GROUP BY branch_id
           UNION ALL
           SELECT active_branch_id,updated_at FROM spaces_conversations WHERE id=$1
         ) GROUP BY branch_id ORDER BY MIN(created_at),branch_id",
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)
}

pub async fn activate(
    pool: &SqlitePool,
    conversation_id: &str,
    branch_id: &str,
) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    let active: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_runs r WHERE r.completed_at IS NULL AND (
         EXISTS (SELECT 1 FROM spaces_builtin_agent_tasks t WHERE t.run_id=r.id AND t.conversation_id=$1)
         OR EXISTS (SELECT 1 FROM spaces_external_agent_conversations e WHERE e.run_id=r.id
           AND e.conversation_id=$1))",
    )
    .bind(conversation_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if active != 0 {
        return Err("conversation-busy".into());
    }
    let updated = sqlx::query(
        "UPDATE spaces_conversations SET active_branch_id=$1,updated_at=$2
         WHERE id=$3 AND EXISTS (
           SELECT 1 FROM spaces_conversation_messages WHERE conversation_id=$3 AND branch_id=$1)",
    )
    .bind(branch_id)
    .bind(crate::clock::unix_time_millis()?)
    .bind(conversation_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(updated, "conversation-branch-unavailable")?;
    tx.commit().await.map_err(db::sql_error)
}

pub async fn branch(
    pool: &SqlitePool,
    conversation_id: &str,
    source_branch_id: &str,
    through_message_id: &str,
    include_boundary: bool,
) -> Result<String, String> {
    let boundary: (i64, i64) = sqlx::query_as(
        "SELECT created_at,rowid FROM spaces_conversation_messages WHERE id=$1 AND conversation_id=$2
         AND branch_id=$3",
    )
    .bind(through_message_id)
    .bind(conversation_id)
    .bind(source_branch_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("conversation-message-unavailable: branch point is unavailable")?;
    let comparison = if include_boundary { "<=" } else { "<" };
    let query = format!(
        "SELECT role,content,created_at FROM spaces_conversation_messages
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
    super::conversations::ensure_branch_idle(&mut tx, conversation_id, source_branch_id).await?;
    for (index, row) in rows.into_iter().enumerate() {
        sqlx::query(
            "INSERT INTO spaces_conversation_messages(id,conversation_id,branch_id,role,content,created_at)
             VALUES($1,$2,$3,$4,$5,$6)",
        ).bind(uuid::Uuid::new_v4().to_string()).bind(conversation_id).bind(&branch_id)
            .bind(row.try_get::<String, _>("role").map_err(db::sql_error)?)
            .bind(row.try_get::<String, _>("content").map_err(db::sql_error)?)
            .bind(now + i64::try_from(index).map_err(|_| "conversation-branch-limit")?)
            .execute(&mut *tx).await.map_err(db::sql_error)?;
    }
    sqlx::query("UPDATE spaces_conversations SET updated_at=$1,active_branch_id=$2 WHERE id=$3")
        .bind(now + i64::try_from(message_count).map_err(|_| "conversation-branch-limit")?)
        .bind(&branch_id)
        .bind(conversation_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(branch_id)
}
