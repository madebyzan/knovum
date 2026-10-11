use crate::spaces::{db, runs};
use serde_json::{json, Value};
use sqlx::{Row, SqlitePool};

pub(super) async fn complete(
    pool: &SqlitePool,
    operation_id: &str,
    state: &str,
    value: &Value,
) -> Result<(), String> {
    let now = crate::foundation::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    complete_in_transaction(&mut tx, operation_id, state, value, now).await?;
    tx.commit().await.map_err(db::sql_error)?;
    runs::announce_change();
    Ok(())
}

pub(super) async fn complete_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    operation_id: &str,
    state: &str,
    value: &Value,
    now: i64,
) -> Result<(), String> {
    let row = sqlx::query("UPDATE spaces_conversation_tool_operations SET state=$1,output=$2,completed_at=$3 WHERE id=$4 AND state IN ('prepared','dispatching') RETURNING run_id,provider_call_id,wire_name")
        .bind(state).bind(sqlx::types::Json(value)).bind(now).bind(operation_id)
        .fetch_optional(&mut **tx).await.map_err(db::sql_error)?.ok_or("conversation-tool-receipt-state-changed")?;
    let run_id: String = row.try_get("run_id").map_err(db::sql_error)?;
    let location: (String, String) = sqlx::query_as("SELECT conversation_id,branch_id FROM spaces_builtin_agent_tasks WHERE run_id=$1 UNION ALL SELECT conversation_id,branch_id FROM spaces_external_agent_conversations WHERE run_id=$1")
        .bind(&run_id).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
    super::super::repository::append_message_in_transaction(tx, &location.0, &location.1,
        super::super::repository::ConversationMessageRole::Tool,
        &json!({"parts":[{"type":"tool_result","callId":row.try_get::<String,_>("provider_call_id").map_err(db::sql_error)?,"name":row.try_get::<String,_>("wire_name").map_err(db::sql_error)?,"result":value}],"runId":run_id}), now).await?;
    runs::append_event(tx, &run_id, None, None, runs::RunEventKind::ConversationToolFinished, &json!({"toolCallId":row.try_get::<String,_>("provider_call_id").map_err(db::sql_error)?,"title":row.try_get::<String,_>("wire_name").map_err(db::sql_error)?,"status":state}), now).await?;
    if state == "outcome_unknown" {
        sqlx::query("UPDATE spaces_runs SET state='waiting',wait_reason='tool_outcome_unknown',state_version=state_version+1,updated_at=$1 WHERE id=$2 AND completed_at IS NULL AND cancel_requested_at IS NULL AND pause_requested_at IS NULL")
            .bind(now).bind(&run_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

pub(super) fn failure(message: impl Into<String>) -> Value {
    json!({"isError":true,"content":[{"type":"text","text":message.into()}]})
}
