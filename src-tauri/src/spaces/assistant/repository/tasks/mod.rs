use serde_json::Value;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};

use crate::{
    agent_runtime::ControlCheckpoint,
    spaces::{assistant::task::NewAssistantTask, db},
};

mod operations;

pub use operations::{
    complete_tool_call, complete_turn, save_checkpoint, start_tool_call, start_turn,
};

pub struct ToolCallRecord {
    pub id: String,
    pub result: Option<Value>,
}

pub struct RecoverableTask {
    pub task_id: String,
    pub run_id: String,
    pub conversation_id: String,
    pub branch_id: String,
    pub grant_id: String,
    pub space_id: String,
    pub mode: super::super::AssistantMode,
    pub profile_version: u16,
    pub prompt: String,
    pub model_binding: crate::ai::models::ExecutionTargetSelection,
    pub catalog_digest: String,
    pub checkpoint: ControlCheckpoint,
    pub max_total_tokens: u32,
    pub max_duration_seconds: u32,
    pub requires_vision: bool,
}

pub async fn recoverable_tasks(pool: &SqlitePool) -> Result<Vec<RecoverableTask>, String> {
    let rows = sqlx::query(
        "SELECT t.id task_id,t.run_id,t.conversation_id,t.branch_id,t.grant_id,t.model_binding,\
         t.catalog_digest,r.home_space_id,r.policy_snapshot,r.input_snapshot,r.budget,c.envelope \
         FROM spaces_assistant_tasks t JOIN spaces_runs r ON r.id=t.run_id \
         JOIN spaces_assistant_checkpoints c ON c.task_id=t.id \
         WHERE r.completed_at IS NULL AND r.state IN ('running','queued','waiting') ",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter().map(parse_recoverable).collect()
}

pub async fn recoverable_task(pool: &SqlitePool, run_id: &str) -> Result<RecoverableTask, String> {
    let row = sqlx::query(
        "SELECT t.id task_id,t.run_id,t.conversation_id,t.branch_id,t.grant_id,t.model_binding,\
         t.catalog_digest,r.home_space_id,r.policy_snapshot,r.input_snapshot,r.budget,c.envelope \
         FROM spaces_assistant_tasks t JOIN spaces_runs r ON r.id=t.run_id \
         JOIN spaces_assistant_checkpoints c ON c.task_id=t.id \
         WHERE r.id=$1 AND r.completed_at IS NULL AND r.state IN ('running','queued','waiting')",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("assistant-task-not-resumable: task is unavailable or not resumable")?;
    parse_recoverable(row)
}

fn parse_recoverable(row: sqlx::sqlite::SqliteRow) -> Result<RecoverableTask, String> {
    let policy = parse_value(&row, "policy_snapshot")?;
    let input = parse_value(&row, "input_snapshot")?;
    let budget = parse_value(&row, "budget")?;
    Ok(RecoverableTask {
        task_id: row.try_get("task_id").map_err(db::sql_error)?,
        run_id: row.try_get("run_id").map_err(db::sql_error)?,
        conversation_id: row.try_get("conversation_id").map_err(db::sql_error)?,
        branch_id: row.try_get("branch_id").map_err(db::sql_error)?,
        grant_id: row.try_get("grant_id").map_err(db::sql_error)?,
        space_id: row.try_get("home_space_id").map_err(db::sql_error)?,
        mode: serde_json::from_value(
            policy
                .get("mode")
                .cloned()
                .ok_or("assistant-mode-missing")?,
        )
        .map_err(db::json_error)?,
        profile_version: policy
            .get("assistantProfileVersion")
            .and_then(Value::as_u64)
            .and_then(|value| u16::try_from(value).ok())
            .ok_or("assistant-profile-missing")?,
        prompt: input
            .get("prompt")
            .and_then(Value::as_str)
            .ok_or("assistant-prompt-missing")?
            .to_owned(),
        model_binding: serde_json::from_str(
            &row.try_get::<String, _>("model_binding")
                .map_err(db::sql_error)?,
        )
        .map_err(db::json_error)?,
        catalog_digest: row.try_get("catalog_digest").map_err(db::sql_error)?,
        checkpoint: serde_json::from_str(
            &row.try_get::<String, _>("envelope")
                .map_err(db::sql_error)?,
        )
        .map_err(db::json_error)?,
        max_total_tokens: budget
            .get("maxTotalTokens")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or("assistant-budget-missing")?,
        max_duration_seconds: budget
            .get("maxDurationSeconds")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or("assistant-duration-budget-missing")?,
        requires_vision: input
            .pointer("/contextManifest")
            .and_then(Value::as_array)
            .is_some_and(|items| {
                items
                    .iter()
                    .any(|item| item.get("kind").and_then(Value::as_str) == Some("image"))
            }),
    })
}

fn parse_value(row: &sqlx::sqlite::SqliteRow, field: &str) -> Result<Value, String> {
    serde_json::from_str(&row.try_get::<String, _>(field).map_err(db::sql_error)?)
        .map_err(db::json_error)
}

pub async fn insert_task(
    tx: &mut Transaction<'_, Sqlite>,
    task: &NewAssistantTask<'_>,
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO spaces_assistant_tasks(id,run_id,conversation_id,branch_id,grant_id,
         model_binding,catalog_digest,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$8)",
    )
    .bind(task.task_id)
    .bind(task.run_id)
    .bind(task.conversation_id)
    .bind(task.branch_id)
    .bind(task.grant_id)
    .bind(task.model_binding)
    .bind(task.catalog_digest)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}
