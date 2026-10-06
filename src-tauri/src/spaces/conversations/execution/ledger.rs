use serde_json::Value;
use sqlx::SqlitePool;

use crate::{
    agent_runtime::protocol::models::{
        AgentModelAdmission, AgentModelCompletion, AgentModelFailure,
    },
    spaces::db,
};

pub(super) async fn admit(
    pool: &SqlitePool,
    task_id: &str,
    request: AgentModelAdmission,
) -> Result<Value, String> {
    let reserved = i64::try_from(request.estimated_input_tokens)
        .map_err(|_| "builtin-agent-model-input-limit")?
        .checked_add(i64::from(request.max_output_tokens))
        .ok_or("builtin-agent-model-input-limit")?;
    let checkpoint = request
        .checkpoint_id
        .ok_or("builtin-agent-model-input-not-durable")?;
    let mut tx = db::begin_write(pool).await?;
    let allowed: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM spaces_runs r WHERE r.id=$1 AND r.state IN ('running','waiting') AND r.completed_at IS NULL
        AND r.cancel_requested_at IS NULL AND r.pause_requested_at IS NULL
        AND (SELECT COUNT(*) FROM spaces_builtin_agent_model_calls WHERE task_id=$2) < json_extract(r.budget,'$.maxTurns')
        AND NOT EXISTS (SELECT 1 FROM spaces_builtin_agent_model_calls WHERE task_id=$2 AND state IN ('outcome_unknown','usage_unknown') AND retry_authorized_at IS NULL)
        AND r.total_tokens+$3+COALESCE((SELECT SUM(estimated_input_tokens+max_output_tokens) FROM spaces_builtin_agent_model_calls WHERE task_id=$2 AND state IN ('requesting','outcome_unknown','usage_unknown')),0) <= json_extract(r.budget,'$.maxTotalTokens')")
        .bind(&request.identity.run_id).bind(task_id).bind(reserved).fetch_optional(&mut *tx).await.map_err(db::sql_error)?;
    allowed
        .ok_or("builtin-agent-model-admission-denied: execution or model budget is unavailable")?;
    sqlx::query("INSERT INTO spaces_builtin_agent_model_calls(id,task_id,purpose,graph_namespace,checkpoint_id,estimated_input_tokens,max_output_tokens,state,created_at)
        VALUES($1,$2,$3,$4,$5,$6,$7,'requesting',$8)")
        .bind(request.request_id).bind(task_id).bind(serde_json::to_value(request.purpose).map_err(db::json_error)?.as_str().ok_or("builtin-agent-purpose-invalid")?)
        .bind(request.graph_namespace.rsplit_once('|').map(|(scope, _)| scope).unwrap_or("")).bind(checkpoint).bind(i64::try_from(request.estimated_input_tokens).map_err(|_| "builtin-agent-model-input-limit")?).bind(i64::from(request.max_output_tokens)).bind(crate::clock::unix_time_millis()?)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(Value::Null)
}

pub(super) async fn complete(
    pool: &SqlitePool,
    task_id: &str,
    request: AgentModelCompletion,
) -> Result<Value, String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let input = request
        .input_tokens
        .map(i64::try_from)
        .transpose()
        .map_err(|_| "builtin-agent-usage-invalid")?;
    let output = request
        .output_tokens
        .map(i64::try_from)
        .transpose()
        .map_err(|_| "builtin-agent-usage-invalid")?;
    let state = if input.is_some() && output.is_some() {
        "received"
    } else {
        "usage_unknown"
    };
    let changed = sqlx::query("UPDATE spaces_builtin_agent_model_calls SET input_tokens=$1,output_tokens=$2,message_id=$3,state=$4,completed_at=$5 WHERE id=$6 AND task_id=$7 AND state IN ('requesting','outcome_unknown')")
        .bind(input).bind(output).bind(request.message_id).bind(state).bind(now).bind(request.request_id).bind(task_id)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    db::require_one(changed, "builtin-agent-model-receipt-unavailable")?;
    if let (Some(input), Some(output)) = (input, output) {
        let total = input
            .checked_add(output)
            .ok_or("builtin-agent-usage-invalid")?;
        // Fees already incurred remain recorded after cancellation and even when
        // the observed total exceeds the admission budget.
        crate::spaces::runs::record_tokens_in_transaction(
            &mut tx,
            &request.identity.run_id,
            input,
            output,
            total,
        )
        .await?;
    }
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(Value::Null)
}

pub(super) async fn fail(
    pool: &SqlitePool,
    task_id: &str,
    request: AgentModelFailure,
) -> Result<Value, String> {
    sqlx::query("UPDATE spaces_builtin_agent_model_calls SET state='outcome_unknown' WHERE id=$1 AND task_id=$2 AND state='requesting'")
        .bind(request.request_id).bind(task_id).execute(pool).await.map_err(db::sql_error)?;
    Ok(Value::Null)
}

pub(in crate::spaces::conversations) async fn require_reconciled(
    pool: &SqlitePool,
    task_id: &str,
) -> Result<(), String> {
    let uncertain: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_builtin_agent_model_calls WHERE task_id IN (SELECT id FROM spaces_builtin_agent_tasks WHERE segment_id=(SELECT segment_id FROM spaces_builtin_agent_tasks WHERE id=$1)) AND state!='committed' AND retry_authorized_at IS NULL")
        .bind(task_id).fetch_one(pool).await.map_err(db::sql_error)?;
    if uncertain != 0 {
        return Err("model-outcome-unknown: authorize a new paid attempt before continuing".into());
    }
    Ok(())
}

pub(super) async fn admit_tool(
    pool: &SqlitePool,
    task_id: &str,
    run_id: &str,
    key: &str,
    name: &str,
) -> Result<Value, String> {
    let mut tx = db::begin_write(pool).await?;
    let previous: Option<String> = sqlx::query_scalar(
        "SELECT name FROM spaces_builtin_agent_tool_admissions WHERE id=$1 AND task_id=$2",
    )
    .bind(key)
    .bind(task_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if let Some(previous) = previous {
        if previous != name {
            return Err("builtin-agent-tool-identity-conflict".into());
        }
        return Ok(Value::Null);
    }
    let allowed: Option<i64> = sqlx::query_scalar("SELECT 1 FROM spaces_runs r WHERE r.id=$1 AND completed_at IS NULL AND cancel_requested_at IS NULL AND pause_requested_at IS NULL
        AND r.total_tokens <= json_extract(r.budget,'$.maxTotalTokens')
        AND NOT EXISTS (SELECT 1 FROM spaces_builtin_agent_model_calls WHERE task_id=$2 AND state IN ('outcome_unknown','usage_unknown') AND retry_authorized_at IS NULL)
        AND (SELECT COUNT(*) FROM spaces_builtin_agent_tool_admissions WHERE task_id=$2) < json_extract(r.budget,'$.maxTools')")
        .bind(run_id).bind(task_id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?;
    allowed.ok_or("builtin-agent-tool-budget-exhausted")?;
    sqlx::query("INSERT INTO spaces_builtin_agent_tool_admissions(id,task_id,name,created_at) VALUES($1,$2,$3,$4)")
        .bind(key).bind(task_id).bind(name).bind(crate::clock::unix_time_millis()?).execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(Value::Null)
}
