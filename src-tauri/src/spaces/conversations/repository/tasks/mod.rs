use serde_json::Value;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};

use crate::{
    agent_runtime::protocol::messages::AgentInputMessage,
    ai::models::{ModelExecutionTarget, ModelTargetSelection},
    spaces::{
        conversations::{builtin_agent::NewBuiltinAgentTask, execution::session::Session},
        db,
    },
};

pub(in crate::spaces::conversations) struct RecoverableTask {
    pub task_id: String,
    pub run_id: String,
    pub conversation_id: String,
    pub branch_id: String,
    pub grant_id: String,
    pub space_id: String,
    pub profile_version: u16,
    pub prompt: String,
    pub model_binding: ModelTargetSelection,
    pub frozen_target: ModelExecutionTarget,
    pub model_profile: crate::ai::settings::contracts::BuiltinAgentModelSettings,
    pub catalog_digest: String,
    pub session: Session,
    pub messages: Vec<AgentInputMessage>,
    pub extensions: crate::extensions::contracts::AgentExtensionSnapshot,
    pub max_turns: u16,
    pub max_tool_calls: u16,
    pub max_total_tokens: u32,
    pub max_duration_seconds: u32,
}

const RECOVERY_SELECT: &str = "SELECT t.id task_id,t.run_id,t.conversation_id,t.branch_id,t.grant_id,t.model_binding,t.catalog_digest,t.input_id,t.control_epoch,
    r.scope_kind,r.home_space_id,r.policy_snapshot,r.input_snapshot,r.budget,s.id segment_id,s.execution_revision,s.graph_version,s.frozen_target,s.model_profile,s.projection_checkpoint_id
    FROM spaces_builtin_agent_tasks t JOIN spaces_runs r ON r.id=t.run_id JOIN spaces_builtin_agent_execution_segments s ON s.id=t.segment_id";

pub(in crate::spaces::conversations) async fn recoverable_tasks(
    pool: &SqlitePool,
) -> Result<Vec<RecoverableTask>, String> {
    let mut query = sqlx::QueryBuilder::<Sqlite>::new(RECOVERY_SELECT);
    query.push(" WHERE r.completed_at IS NULL AND r.cancel_requested_at IS NULL AND r.state IN ('running','queued','waiting') ORDER BY r.created_at");
    let rows = query.build().fetch_all(pool).await.map_err(db::sql_error)?;
    rows.into_iter().map(parse_recoverable).collect()
}

pub(in crate::spaces::conversations) async fn recoverable_task(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<RecoverableTask, String> {
    let mut query = sqlx::QueryBuilder::<Sqlite>::new(RECOVERY_SELECT);
    query.push(" WHERE r.id=$1 AND r.completed_at IS NULL AND r.state IN ('running','queued','waiting','paused')");
    let row = query
        .build()
        .bind(run_id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?
        .ok_or("builtin-agent-task-not-resumable")?;
    parse_recoverable(row)
}

fn parse_recoverable(row: sqlx::sqlite::SqliteRow) -> Result<RecoverableTask, String> {
    let policy = parse_value(&row, "policy_snapshot")?;
    let input = parse_value(&row, "input_snapshot")?;
    let budget = parse_value(&row, "budget")?;
    if row
        .try_get::<String, _>("graph_version")
        .map_err(db::sql_error)?
        != crate::agent_runtime::protocol::GRAPH_VERSION
    {
        return Err("builtin-agent-graph-version-changed: start a new execution segment".into());
    }
    let scope_kind: String = row.try_get("scope_kind").map_err(db::sql_error)?;
    let scope = crate::spaces::authorization::RunScope::from_db(
        &scope_kind,
        row.try_get("home_space_id").map_err(db::sql_error)?,
    )?;
    Ok(RecoverableTask {
        task_id: row.try_get("task_id").map_err(db::sql_error)?,
        run_id: row.try_get("run_id").map_err(db::sql_error)?,
        conversation_id: row.try_get("conversation_id").map_err(db::sql_error)?,
        branch_id: row.try_get("branch_id").map_err(db::sql_error)?,
        grant_id: row.try_get("grant_id").map_err(db::sql_error)?,
        space_id: scope.require_space_id()?.to_owned(),
        profile_version: u16::try_from(number(&policy, "builtinAgentProfileVersion")?)
            .map_err(|_| "builtin-agent-profile-invalid")?,
        prompt: input["prompt"]
            .as_str()
            .ok_or("builtin-agent-prompt-missing")?
            .into(),
        model_binding: serde_json::from_value(parse_value(&row, "model_binding")?)
            .map_err(db::json_error)?,
        frozen_target: serde_json::from_value(parse_value(&row, "frozen_target")?)
            .map_err(db::json_error)?,
        model_profile: serde_json::from_value(parse_value(&row, "model_profile")?)
            .map_err(db::json_error)?,
        catalog_digest: row.try_get("catalog_digest").map_err(db::sql_error)?,
        session: Session {
            thread_id: row.try_get("segment_id").map_err(db::sql_error)?,
            execution_revision: row.try_get("execution_revision").map_err(db::sql_error)?,
            control_epoch: u64::try_from(
                row.try_get::<i64, _>("control_epoch")
                    .map_err(db::sql_error)?,
            )
            .map_err(|_| "builtin-agent-epoch-invalid")?,
            projection_checkpoint_id: row
                .try_get("projection_checkpoint_id")
                .map_err(db::sql_error)?,
            input_id: row.try_get("input_id").map_err(db::sql_error)?,
        },
        messages: serde_json::from_value(input["messages"].clone()).map_err(db::json_error)?,
        extensions: serde_json::from_value(input["extensions"].clone()).map_err(db::json_error)?,
        max_turns: u16::try_from(number(&budget, "maxTurns")?)
            .map_err(|_| "builtin-agent-budget-invalid")?,
        max_tool_calls: u16::try_from(number(&budget, "maxTools")?)
            .map_err(|_| "builtin-agent-budget-invalid")?,
        max_total_tokens: u32::try_from(number(&budget, "maxTotalTokens")?)
            .map_err(|_| "builtin-agent-budget-invalid")?,
        max_duration_seconds: u32::try_from(number(&budget, "maxDurationSeconds")?)
            .map_err(|_| "builtin-agent-budget-invalid")?,
    })
}

fn number(value: &Value, key: &str) -> Result<u64, String> {
    value[key]
        .as_u64()
        .ok_or_else(|| format!("builtin-agent-execution-field-missing: {key}"))
}
fn parse_value(row: &sqlx::sqlite::SqliteRow, field: &str) -> Result<Value, String> {
    serde_json::from_str(&row.try_get::<String, _>(field).map_err(db::sql_error)?)
        .map_err(db::json_error)
}

pub async fn insert_task(
    tx: &mut Transaction<'_, Sqlite>,
    task: &NewBuiltinAgentTask<'_>,
    now: i64,
) -> Result<(), String> {
    sqlx::query("INSERT INTO spaces_builtin_agent_tasks(id,run_id,conversation_id,branch_id,grant_id,model_binding,catalog_digest,segment_id,input_id,created_at,updated_at)
        VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$10)")
        .bind(task.task_id).bind(task.run_id).bind(task.conversation_id).bind(task.branch_id).bind(task.grant_id).bind(sqlx::types::Json(task.model_binding)).bind(task.catalog_digest)
        .bind(&task.session.thread_id).bind(&task.session.input_id).bind(now).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}
