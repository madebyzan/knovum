use crate::spaces::{db, runs};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use tauri::AppHandle;
use ts_rs::TS;

#[derive(Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct ConversationToolOperation {
    pub id: String,
    pub run_id: String,
    pub name: String,
    pub source: crate::extensions::contracts::AgentToolBinding,
    pub arguments: Value,
    pub state: String,
    pub approval: Option<String>,
    pub result: Option<Value>,
    pub resolution: Option<String>,
    pub attempt: i64,
}

#[derive(Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum ToolOperationDecision {
    Approve,
    Deny,
    NewAttempt,
    Stop,
}

#[tauri::command]
pub async fn list_conversation_tool_operations(
    app: AppHandle,
    run_id: String,
) -> Result<Vec<ConversationToolOperation>, String> {
    let pool = db::pool(&app).await?;
    let rows = sqlx::query("SELECT id,run_id,wire_name,source,arguments,state,approval,result,resolution,attempt FROM spaces_conversation_tool_operations WHERE run_id=$1 ORDER BY created_at,attempt").bind(run_id).fetch_all(pool).await.map_err(db::sql_error)?;
    rows.iter()
        .map(|row| {
            Ok(ConversationToolOperation {
                id: row.try_get("id").map_err(db::sql_error)?,
                run_id: row.try_get("run_id").map_err(db::sql_error)?,
                name: row.try_get("wire_name").map_err(db::sql_error)?,
                source: serde_json::from_str(
                    &row.try_get::<String, _>("source").map_err(db::sql_error)?,
                )
                .map_err(db::json_error)?,
                arguments: serde_json::from_str(
                    &row.try_get::<String, _>("arguments")
                        .map_err(db::sql_error)?,
                )
                .map_err(db::json_error)?,
                state: row.try_get("state").map_err(db::sql_error)?,
                approval: row.try_get("approval").map_err(db::sql_error)?,
                result: row
                    .try_get::<Option<String>, _>("result")
                    .map_err(db::sql_error)?
                    .map(|result| serde_json::from_str(&result).map_err(db::json_error))
                    .transpose()?,
                resolution: row.try_get("resolution").map_err(db::sql_error)?,
                attempt: row.try_get("attempt").map_err(db::sql_error)?,
            })
        })
        .collect()
}

#[tauri::command]
pub async fn decide_conversation_tool_operation(
    app: AppHandle,
    operation_id: String,
    decision: ToolOperationDecision,
) -> Result<(), String> {
    let pool = db::pool(&app).await?;
    let run_id = super::decisions::apply(pool, &operation_id, decision).await?;
    runs::announce_change();
    if matches!(decision, ToolOperationDecision::Stop) {
        return runs::cancel_run(app, run_id)
            .await
            .map_err(|error| error.to_string());
    }
    let run = runs::load_run(pool, &run_id).await?;
    let (pending, pause_clear): (i64, bool) = sqlx::query_as("SELECT (SELECT count(*) FROM spaces_conversation_tool_operations WHERE run_id=r.id AND ((state='prepared' AND approval IS NULL) OR (state='outcome_unknown' AND resolution IS NULL))), pause_requested_at IS NULL FROM spaces_runs r WHERE id=$1")
        .bind(&run_id).fetch_one(pool).await.map_err(db::sql_error)?;
    if pending == 0
        && pause_clear
        && run.kind == runs::RunKind::BuiltinAgent
        && run.state == runs::RunState::Waiting
    {
        super::super::builtin_agent::resume(app, &run_id).await?;
    }
    Ok(())
}

pub(in crate::spaces) async fn wait(
    pool: &sqlx::SqlitePool,
    run_id: &str,
    operation_id: &str,
) -> Result<(), String> {
    let mut changes = runs::subscribe_to_changes();
    loop {
        let run = runs::load_run(pool, run_id).await?;
        if run.state.is_terminal() || run.state == runs::RunState::Paused {
            return Err("conversation-tool-run-inactive".into());
        }
        let row: (Option<String>, Option<String>, String) = sqlx::query_as(
            "SELECT approval,resolution,state FROM spaces_conversation_tool_operations WHERE id=$1",
        )
        .bind(operation_id)
        .fetch_one(pool)
        .await
        .map_err(db::sql_error)?;
        if row.0.is_some() && row.2 != "outcome_unknown" || row.1.as_deref() == Some("new_attempt")
        {
            return Ok(());
        }
        changes
            .changed()
            .await
            .map_err(|_| "run-observation-closed")?;
    }
}
