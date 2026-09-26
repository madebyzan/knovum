use sqlx::{Row, SqlitePool};

use crate::spaces::db;

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
