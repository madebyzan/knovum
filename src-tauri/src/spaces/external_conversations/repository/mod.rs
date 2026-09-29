use sqlx::{Row, SqlitePool};

use crate::spaces::db;

pub async fn start_turn(
    pool: &SqlitePool,
    conversation_id: &str,
    prompt: &str,
    attachments: &[crate::spaces::assistant::AssistantMessageAttachment],
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let binding = sqlx::query(
        "SELECT state,assistant_conversation_id,branch_id,run_id
         FROM spaces_external_agent_conversations WHERE id=$1",
    )
    .bind(conversation_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("external-conversation-unavailable")?;
    if binding
        .try_get::<String, _>("state")
        .map_err(db::sql_error)?
        != "connected"
    {
        return Err("external-conversation-disconnected: start a new session to continue".into());
    }
    let assistant_id: String = binding
        .try_get("assistant_conversation_id")
        .map_err(db::sql_error)?;
    let branch_id: String = binding.try_get("branch_id").map_err(db::sql_error)?;
    let run_id: String = binding.try_get("run_id").map_err(db::sql_error)?;
    let assistant_message_id = uuid::Uuid::new_v4().to_string();
    append_message(
        &mut tx,
        &assistant_id,
        &branch_id,
        "user",
        prompt,
        attachments,
        &run_id,
        now,
    )
    .await?;
    sqlx::query(
        "INSERT INTO spaces_assistant_messages(id,conversation_id,branch_id,role,content,created_at)
         VALUES($1,$2,$3,'assistant',$4,$5)",
    )
    .bind(&assistant_message_id)
    .bind(&assistant_id)
    .bind(&branch_id)
    .bind(serde_json::json!({"parts":[{"type":"text","text":""}], "runId": run_id}).to_string())
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "INSERT INTO spaces_external_agent_turns(id,conversation_id,assistant_message_id,state,created_at)
         VALUES($1,$2,$3,'running',$4)",
    )
    .bind(&id)
    .bind(conversation_id)
    .bind(&assistant_message_id)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_external_agent_conversations SET updated_at=$1 WHERE id=$2")
        .bind(now)
        .bind(conversation_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_assistant_conversations SET updated_at=$1 WHERE id=$2")
        .bind(now)
        .bind(&assistant_id)
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
) -> Result<(), String> {
    let message_id: Option<String> = sqlx::query_scalar(
        "SELECT assistant_message_id FROM spaces_external_agent_turns
         WHERE conversation_id=$1 AND state='running'
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(conversation_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if let Some(message_id) = message_id {
        sqlx::query(
            "UPDATE spaces_assistant_messages SET content=json_set(content,'$.parts[0].text',
             json_extract(content,'$.parts[0].text') || $1) WHERE id=$2",
        )
        .bind(content)
        .bind(message_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    Ok(())
}

pub async fn clear_agent_message_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    conversation_id: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE spaces_assistant_messages SET content=json_set(content,'$.parts[0].text','')
         WHERE id=(SELECT assistant_message_id FROM spaces_external_agent_turns
           WHERE conversation_id=$1 AND state='running' ORDER BY created_at DESC LIMIT 1)",
    )
    .bind(conversation_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
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
    sqlx::query(
        "DELETE FROM spaces_assistant_messages WHERE id=(
           SELECT assistant_message_id FROM spaces_external_agent_turns WHERE id=$1
         ) AND json_extract(content,'$.parts[0].text')=''",
    )
    .bind(turn_id)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub async fn has_turn_output(pool: &SqlitePool, turn_id: &str) -> Result<bool, String> {
    let has_output: i64 = sqlx::query_scalar(
        "SELECT EXISTS(
           SELECT 1 FROM spaces_external_agent_turns t
           JOIN spaces_external_agent_conversations c ON c.id=t.conversation_id
           WHERE t.id=$1 AND (
             EXISTS(SELECT 1 FROM spaces_assistant_messages m
               WHERE m.id=t.assistant_message_id
                 AND TRIM(json_extract(m.content,'$.parts[0].text'))<>'')
             OR EXISTS(SELECT 1 FROM spaces_run_events e
               WHERE e.run_id=c.run_id AND e.created_at>=t.created_at
                 AND e.kind IN ('external_agent_tool_started','external_agent_tool_updated')
                 AND json_extract(e.payload,'$.status')='completed')
           ))",
    )
    .bind(turn_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(has_output != 0)
}

async fn append_message(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    conversation_id: &str,
    branch_id: &str,
    role: &str,
    content: &str,
    attachments: &[crate::spaces::assistant::AssistantMessageAttachment],
    run_id: &str,
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO spaces_assistant_messages(
         id,conversation_id,branch_id,role,content,created_at) VALUES($1,$2,$3,$4,$5,$6)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(conversation_id)
    .bind(branch_id)
    .bind(role)
    .bind(
        serde_json::json!({"parts":[{"type":"text","text":content}], "attachments": attachments, "runId": run_id})
            .to_string(),
    )
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}
