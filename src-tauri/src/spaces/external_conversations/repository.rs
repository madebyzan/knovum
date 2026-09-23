use sqlx::SqlitePool;

use super::{ExternalConversationMessage, ExternalConversationSummary};
use crate::spaces::db;

mod rows;
mod session_state;

pub use session_state::{update_session_mode, update_session_options};

pub async fn list(
    pool: &SqlitePool,
    space_id: &str,
) -> Result<Vec<ExternalConversationSummary>, String> {
    let rows = sqlx::query(
        "SELECT * FROM spaces_external_agent_conversations WHERE space_id=$1 ORDER BY updated_at DESC",
    ).bind(space_id).fetch_all(pool).await.map_err(db::sql_error)?;
    rows.into_iter().map(rows::conversation).collect()
}

pub async fn list_messages(
    pool: &SqlitePool,
    conversation_id: &str,
) -> Result<Vec<ExternalConversationMessage>, String> {
    let rows = sqlx::query(
        "SELECT * FROM spaces_external_agent_messages WHERE conversation_id=$1
         ORDER BY created_at,id LIMIT 1000",
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter().map(rows::message).collect()
}

pub async fn start_turn(
    pool: &SqlitePool,
    conversation_id: &str,
    prompt: &str,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let state: Option<String> =
        sqlx::query_scalar("SELECT state FROM spaces_external_agent_conversations WHERE id=$1")
            .bind(conversation_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db::sql_error)?;
    if state.as_deref() != Some("connected") {
        return Err("external-conversation-disconnected: start a new session to continue".into());
    }
    sqlx::query(
        "INSERT INTO spaces_external_agent_turns(id,conversation_id,state,created_at)
         VALUES($1,$2,'running',$3)",
    )
    .bind(&id)
    .bind(conversation_id)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    append_message(&mut tx, conversation_id, &id, "user", prompt, now).await?;
    sqlx::query("UPDATE spaces_external_agent_conversations SET updated_at=$1 WHERE id=$2")
        .bind(now)
        .bind(conversation_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(id)
}

pub async fn append_agent_message_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    conversation_id: &str,
    content: &str,
    now: i64,
) -> Result<(), String> {
    let turn_id: Option<String> = sqlx::query_scalar(
        "SELECT id FROM spaces_external_agent_turns WHERE conversation_id=$1 AND state='running'
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(conversation_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if let Some(turn_id) = turn_id {
        let updated = sqlx::query(
            "UPDATE spaces_external_agent_messages SET content=content || $1
             WHERE id=(SELECT id FROM spaces_external_agent_messages
               WHERE conversation_id=$2 AND turn_id=$3 AND role='assistant'
               ORDER BY created_at DESC,id DESC LIMIT 1)",
        )
        .bind(content)
        .bind(conversation_id)
        .bind(&turn_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        if updated.rows_affected() == 0 {
            append_message(tx, conversation_id, &turn_id, "assistant", content, now).await?;
        }
    }
    Ok(())
}

pub async fn finish_turn(
    pool: &SqlitePool,
    turn_id: &str,
    state: &str,
    error: Option<&str>,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    sqlx::query(
        "UPDATE spaces_external_agent_turns SET state=$1,error=$2,completed_at=$3
         WHERE id=$4 AND state='running'",
    )
    .bind(state)
    .bind(error)
    .bind(now)
    .bind(turn_id)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

async fn append_message(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    conversation_id: &str,
    turn_id: &str,
    role: &str,
    content: &str,
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO spaces_external_agent_messages(
         id,conversation_id,turn_id,role,content,created_at) VALUES($1,$2,$3,$4,$5,$6)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(conversation_id)
    .bind(turn_id)
    .bind(role)
    .bind(content)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}
