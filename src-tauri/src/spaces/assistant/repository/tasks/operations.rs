use serde_json::Value;
use sqlx::{Row, SqlitePool};

use super::AssistantToolCallOutcome;
use crate::spaces::db;

pub struct AssistantToolCallStart<'a> {
    pub task_id: &'a str,
    pub run_id: &'a str,
    pub graph_namespace: &'a str,
    pub message_id: &'a str,
    pub provider_call_id: &'a str,
    pub wire_name: &'a str,
    pub action_id: &'a str,
    pub invocation_id: &'a str,
    pub arguments: &'a Value,
    pub target_selection: Option<Value>,
}

pub struct AssistantToolCallCompletion<'a> {
    pub id: &'a str,
    pub run_id: &'a str,
    pub conversation_id: &'a str,
    pub branch_id: &'a str,
    pub call_id: &'a str,
    pub name: &'a str,
    pub tool_output: &'a Value,
}

pub async fn start_tool_call(
    pool: &SqlitePool,
    call: AssistantToolCallStart<'_>,
) -> Result<AssistantToolCallOutcome, String> {
    let AssistantToolCallStart {
        task_id,
        run_id,
        graph_namespace,
        message_id,
        provider_call_id,
        wire_name,
        action_id,
        invocation_id,
        arguments,
        target_selection,
    } = call;
    let id = uuid::Uuid::new_v4().to_string();
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let inserted = sqlx::query("INSERT OR IGNORE INTO spaces_assistant_tool_operations(id,task_id,graph_namespace,message_id,provider_call_id,
         wire_name,action_id,invocation_id,target_selection,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)")
        .bind(&id).bind(task_id).bind(graph_namespace).bind(message_id).bind(provider_call_id).bind(wire_name)
        .bind(action_id).bind(invocation_id).bind(target_selection.as_ref().map(serde_json::to_string).transpose().map_err(db::json_error)?).bind(now)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    if inserted.rows_affected() != 0 {
        crate::spaces::runs::append_event(
            &mut tx,
            run_id,
            None,
            None,
            crate::spaces::runs::RunEventKind::AssistantToolStarted,
            &serde_json::json!({
                "toolCallId": provider_call_id,
                "actionId": action_id,
                "context": arguments,
            }),
            now,
        )
        .await?;
    }
    tx.commit().await.map_err(db::sql_error)?;
    if inserted.rows_affected() != 0 {
        crate::spaces::runs::announce_change();
    }
    let row = sqlx::query("SELECT id,result,target_selection FROM spaces_assistant_tool_operations WHERE invocation_id=$1")
        .bind(invocation_id).fetch_one(pool).await.map_err(db::sql_error)?;
    let stored_id: String = row.try_get("id").map_err(db::sql_error)?;
    let tool_output = row
        .try_get::<Option<String>, _>("result")
        .map_err(db::sql_error)?
        .map(|value| serde_json::from_str(&value).map_err(db::json_error))
        .transpose()?;
    Ok(AssistantToolCallOutcome {
        id: stored_id,
        output: tool_output,
        target_selection: row
            .try_get::<Option<String>, _>("target_selection")
            .map_err(db::sql_error)?
            .map(|value| serde_json::from_str(&value).map_err(db::json_error))
            .transpose()?,
    })
}

pub async fn complete_tool_call(
    pool: &SqlitePool,
    call: AssistantToolCallCompletion<'_>,
) -> Result<(), String> {
    let AssistantToolCallCompletion {
        id,
        run_id,
        conversation_id,
        branch_id,
        call_id,
        name,
        tool_output,
    } = call;
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let updated = sqlx::query("UPDATE spaces_assistant_tool_operations SET result=$1,completed_at=$2 WHERE id=$3 AND completed_at IS NULL")
        .bind(serde_json::to_string(tool_output).map_err(db::json_error)?).bind(now).bind(id)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    db::require_one(updated, "assistant-tool-call-already-completed")?;
    super::super::messages::append_message_in_transaction(
        &mut tx, conversation_id, branch_id, super::super::AssistantMessageRole::Tool,
        &serde_json::json!({"parts": [{"type": "tool_result", "callId": call_id, "name": name, "result": tool_output}], "runId": run_id}), now,
    ).await?;
    crate::spaces::runs::append_event(
        &mut tx,
        run_id,
        None,
        None,
        crate::spaces::runs::RunEventKind::AssistantToolFinished,
        &serde_json::json!({"toolCallId": call_id}),
        now,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(())
}
