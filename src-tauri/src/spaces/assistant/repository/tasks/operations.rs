use serde_json::Value;
use sqlx::{Row, SqlitePool};

use super::AssistantToolCallOutcome;
use crate::{agent_runtime::ControlCheckpoint, spaces::db};

pub async fn start_turn(
    pool: &SqlitePool,
    task_id: &str,
    sequence: i64,
    request: &Value,
) -> Result<(String, Option<Value>), String> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT OR IGNORE INTO spaces_assistant_model_turns(id,task_id,sequence,request,state,created_at)
         VALUES($1,$2,$3,$4,'requesting',$5)")
        .bind(&id).bind(task_id).bind(sequence)
        .bind(serde_json::to_string(request).map_err(db::json_error)?)
        .bind(crate::clock::unix_time_millis()?).execute(pool).await.map_err(db::sql_error)?;
    let row = sqlx::query("SELECT id,state,response FROM spaces_assistant_model_turns WHERE task_id=$1 AND sequence=$2")
        .bind(task_id).bind(sequence).fetch_one(pool).await.map_err(db::sql_error)?;
    let stored_id: String = row.try_get("id").map_err(db::sql_error)?;
    let state: String = row.try_get("state").map_err(db::sql_error)?;
    let response: Option<String> = row.try_get("response").map_err(db::sql_error)?;
    if stored_id != id && state == "requesting" {
        sqlx::query("UPDATE spaces_assistant_model_turns SET state='outcome_unknown' WHERE id=$1 AND state='requesting'")
            .bind(&stored_id).execute(pool).await.map_err(db::sql_error)?;
        return Err("model-outcome-unknown: previous request may have reached the provider".into());
    }
    Ok((
        stored_id,
        response
            .map(|value| serde_json::from_str(&value).map_err(db::json_error))
            .transpose()?,
    ))
}

pub async fn complete_turn(pool: &SqlitePool, id: &str, response: &Value) -> Result<(), String> {
    sqlx::query("UPDATE spaces_assistant_model_turns SET response=$1,state='completed',completed_at=$2 WHERE id=$3 AND state='requesting'")
        .bind(serde_json::to_string(response).map_err(db::json_error)?)
        .bind(crate::clock::unix_time_millis()?).bind(id).execute(pool).await.map_err(db::sql_error)?;
    Ok(())
}

pub async fn save_checkpoint(
    pool: &SqlitePool,
    task_id: &str,
    checkpoint: &ControlCheckpoint,
) -> Result<(), String> {
    sqlx::query("INSERT INTO spaces_assistant_checkpoints(task_id,envelope,updated_at) VALUES($1,$2,$3)
         ON CONFLICT(task_id) DO UPDATE SET envelope=excluded.envelope,updated_at=excluded.updated_at")
        .bind(task_id).bind(serde_json::to_string(checkpoint).map_err(db::json_error)?)
        .bind(crate::clock::unix_time_millis()?).execute(pool).await.map_err(db::sql_error)?;
    Ok(())
}

pub async fn start_tool_call(
    pool: &SqlitePool,
    task_id: &str,
    model_turn_id: &str,
    provider_call_id: &str,
    wire_name: &str,
    action_id: &str,
    invocation_id: &str,
) -> Result<AssistantToolCallOutcome, String> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT OR IGNORE INTO spaces_assistant_tool_calls(id,task_id,model_turn_id,provider_call_id,
         wire_name,action_id,invocation_id,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
        .bind(&id).bind(task_id).bind(model_turn_id).bind(provider_call_id).bind(wire_name)
        .bind(action_id).bind(invocation_id).bind(crate::clock::unix_time_millis()?)
        .execute(pool).await.map_err(db::sql_error)?;
    let row = sqlx::query("SELECT id,result FROM spaces_assistant_tool_calls WHERE task_id=$1 AND provider_call_id=$2")
        .bind(task_id).bind(provider_call_id).fetch_one(pool).await.map_err(db::sql_error)?;
    let stored_id: String = row.try_get("id").map_err(db::sql_error)?;
    let tool_output = row
        .try_get::<Option<String>, _>("result")
        .map_err(db::sql_error)?
        .map(|value| serde_json::from_str(&value).map_err(db::json_error))
        .transpose()?;
    Ok(AssistantToolCallOutcome {
        id: stored_id,
        result: tool_output,
    })
}

pub async fn complete_tool_call(
    pool: &SqlitePool,
    id: &str,
    conversation_id: &str,
    branch_id: &str,
    call_id: &str,
    name: &str,
    tool_output: &Value,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let updated = sqlx::query("UPDATE spaces_assistant_tool_calls SET result=$1,completed_at=$2 WHERE id=$3 AND completed_at IS NULL")
        .bind(serde_json::to_string(tool_output).map_err(db::json_error)?).bind(now).bind(id)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    db::require_one(updated, "assistant-tool-call-already-completed")?;
    super::super::messages::append_message_in_transaction(
        &mut tx, conversation_id, branch_id, super::super::AssistantMessageRole::Tool,
        &serde_json::json!({"parts": [{"type": "tool_result", "callId": call_id, "name": name, "result": tool_output}]}), now,
    ).await?;
    tx.commit().await.map_err(db::sql_error)
}
