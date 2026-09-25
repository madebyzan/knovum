use serde_json::{json, Value};
use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

pub(super) async fn rows(
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
        run_rows.push(run(&row)?);
        steps.extend(step_rows(tx, &id).await?);
        attempts.extend(attempt_rows(tx, &id).await?);
        events.extend(event_rows(tx, &id).await?);
    }
    Ok((run_rows, steps, attempts, events))
}

fn run(row: &sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    Ok(json!({
        "id": text(row, "id")?, "parentRunId": optional_text(row, "parent_run_id")?,
        "retryOfRunId": optional_text(row, "retry_of_run_id")?, "forkOfRunId": optional_text(row, "fork_of_run_id")?,
        "principalKind": text(row, "principal_kind")?, "principalId": text(row, "principal_id")?,
        "kind": text(row, "kind")?, "state": text(row, "state")?, "stateVersion": integer(row, "state_version")?,
        "policySnapshot": parsed(row, "policy_snapshot")?, "inputSnapshot": parsed(row, "input_snapshot")?,
        "budget": parsed(row, "budget")?, "inputTokens": integer(row, "input_tokens")?,
        "outputTokens": integer(row, "output_tokens")?, "totalTokens": integer(row, "total_tokens")?,
        "stepAttempts": integer(row, "step_attempts")?,
        "expandedSteps": integer(row, "expanded_steps")?,
        "outputBytes": integer(row, "output_bytes")?,
        "createdAt": integer(row, "created_at")?, "updatedAt": integer(row, "updated_at")?,
        "completedAt": integer(row, "completed_at")?,
    }))
}

async fn step_rows(tx: &mut Transaction<'_, Sqlite>, run_id: &str) -> Result<Vec<Value>, String> {
    let rows = sqlx::query("SELECT * FROM spaces_run_steps WHERE run_id=$1 ORDER BY created_at,id")
        .bind(run_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    rows.iter().map(|row| Ok(json!({
        "id": text(row,"id")?, "runId": run_id, "definitionNodeId": text(row,"definition_node_id")?,
        "instancePath": text(row,"instance_path")?, "nodeKind": text(row,"node_kind")?,
        "dependsOn": parsed(row,"depends_on")?, "configuration": parsed(row,"configuration")?,
        "state": text(row,"state")?, "stateVersion": integer(row,"state_version")?,
        "resolvedInput": parsed(row,"resolved_input")?, "output": optional_parsed(row,"output")?,
        "createdAt": integer(row,"created_at")?, "updatedAt": integer(row,"updated_at")?,
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
        "id": text(row,"id")?, "runId": run_id, "stepInstanceId": text(row,"step_instance_id")?,
        "attemptNumber": integer(row,"attempt_number")?, "state": text(row,"state")?,
        "requestDigest": text(row,"request_digest")?, "idempotencyKey": text(row,"idempotency_key")?,
        "externalTaskId": optional_text(row,"external_task_id")?, "checkpoint": optional_parsed(row,"checkpoint")?,
        "result": optional_parsed(row,"result")?, "errorCode": optional_text(row,"error_code")?,
        "startedAt": integer(row,"started_at")?, "finishedAt": optional_integer(row,"finished_at")?,
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
                "runId": run_id, "stepInstanceId": optional_text(row,"step_instance_id")?,
                "attemptId": optional_text(row,"attempt_id")?, "kind": text(row,"kind")?,
                "payload": parsed(row,"payload")?, "createdAt": integer(row,"created_at")?,
            }))
        })
        .collect()
}

fn text(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<String, String> {
    row.try_get(key).map_err(db::sql_error)
}
fn optional_text(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<Option<String>, String> {
    row.try_get(key).map_err(db::sql_error)
}
fn integer(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<i64, String> {
    row.try_get(key).map_err(db::sql_error)
}
fn optional_integer(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<Option<i64>, String> {
    row.try_get(key).map_err(db::sql_error)
}
fn parsed(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<Value, String> {
    serde_json::from_str(&text(row, key)?).map_err(db::json_error)
}
fn optional_parsed(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<Option<Value>, String> {
    optional_text(row, key)?
        .map(|value| serde_json::from_str(&value).map_err(db::json_error))
        .transpose()
}
