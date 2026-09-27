use super::decode::{
    decode_integer, decode_json, decode_optional_integer, decode_optional_json,
    decode_optional_text, decode_text,
};
use serde_json::{json, Value};
use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

pub(super) async fn load(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<(Vec<Value>, Vec<Value>, Vec<Value>, Vec<Value>), String> {
    let runs = sqlx::query("SELECT * FROM spaces_runs WHERE home_space_id=$1 AND completed_at IS NOT NULL ORDER BY created_at,id")
        .bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    let mut run_rows = Vec::with_capacity(runs.len());
    let mut steps = Vec::new();
    let mut attempts = Vec::new();
    let mut events = Vec::new();
    for row in runs {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        run_rows.push(decode_run_projection(&row)?);
        steps.extend(step_rows(tx, &id).await?);
        attempts.extend(attempt_rows(tx, &id).await?);
        events.extend(event_rows(tx, &id).await?);
    }
    Ok((run_rows, steps, attempts, events))
}

fn decode_run_projection(row: &sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    Ok(json!({
        "id": decode_text(row, "id")?, "parentRunId": decode_optional_text(row, "parent_run_id")?,
        "retryOfRunId": decode_optional_text(row, "retry_of_run_id")?, "forkOfRunId": decode_optional_text(row, "fork_of_run_id")?,
        "principalKind": decode_text(row, "principal_kind")?, "principalId": decode_text(row, "principal_id")?,
        "kind": decode_text(row, "kind")?, "state": decode_text(row, "state")?, "stateVersion": decode_integer(row, "state_version")?,
        "policySnapshot": decode_json(row, "policy_snapshot")?, "inputSnapshot": decode_json(row, "input_snapshot")?,
        "budget": decode_json(row, "budget")?, "inputTokens": decode_integer(row, "input_tokens")?,
        "outputTokens": decode_integer(row, "output_tokens")?, "totalTokens": decode_integer(row, "total_tokens")?,
        "stepAttempts": decode_integer(row, "step_attempts")?,
        "expandedSteps": decode_integer(row, "expanded_steps")?,
        "outputBytes": decode_integer(row, "output_bytes")?,
        "createdAt": decode_integer(row, "created_at")?, "updatedAt": decode_integer(row, "updated_at")?,
        "completedAt": decode_integer(row, "completed_at")?,
    }))
}

async fn step_rows(tx: &mut Transaction<'_, Sqlite>, run_id: &str) -> Result<Vec<Value>, String> {
    let rows = sqlx::query("SELECT * FROM spaces_run_steps WHERE run_id=$1 ORDER BY created_at,id")
        .bind(run_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    rows.iter().map(|row| Ok(json!({
        "id": decode_text(row,"id")?, "runId": run_id, "definitionNodeId": decode_text(row,"definition_node_id")?,
        "instancePath": decode_text(row,"instance_path")?, "nodeKind": decode_text(row,"node_kind")?,
        "dependsOn": decode_json(row,"depends_on")?, "configuration": decode_json(row,"configuration")?,
        "state": decode_text(row,"state")?, "stateVersion": decode_integer(row,"state_version")?,
        "resolvedInput": decode_json(row,"resolved_input")?, "output": decode_optional_json(row,"output")?,
        "createdAt": decode_integer(row,"created_at")?, "updatedAt": decode_integer(row,"updated_at")?,
    }))).collect()
}

async fn attempt_rows(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
) -> Result<Vec<Value>, String> {
    let rows =
        sqlx::query("SELECT * FROM spaces_run_attempts WHERE run_id=$1 ORDER BY started_at,id")
            .bind(run_id)
            .fetch_all(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    rows.iter().map(|row| Ok(json!({
        "id": decode_text(row,"id")?, "runId": run_id, "stepInstanceId": decode_text(row,"step_instance_id")?,
        "attemptNumber": decode_integer(row,"attempt_number")?, "state": decode_text(row,"state")?,
        "requestDigest": decode_text(row,"request_digest")?, "idempotencyKey": decode_text(row,"idempotency_key")?,
        "externalTaskId": decode_optional_text(row,"external_task_id")?, "checkpoint": decode_optional_json(row,"checkpoint")?,
        "result": decode_optional_json(row,"result")?, "errorCode": decode_optional_text(row,"error_code")?,
        "startedAt": decode_integer(row,"started_at")?, "finishedAt": decode_optional_integer(row,"finished_at")?,
    }))).collect()
}

async fn event_rows(tx: &mut Transaction<'_, Sqlite>, run_id: &str) -> Result<Vec<Value>, String> {
    let rows = sqlx::query("SELECT * FROM spaces_run_events WHERE run_id=$1 ORDER BY sequence")
        .bind(run_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    rows.iter()
        .map(|row| {
            Ok(json!({
                "runId": run_id, "stepInstanceId": decode_optional_text(row,"step_instance_id")?,
                "attemptId": decode_optional_text(row,"attempt_id")?, "kind": decode_text(row,"kind")?,
                "payload": decode_json(row,"payload")?, "createdAt": decode_integer(row,"created_at")?,
            }))
        })
        .collect()
}
