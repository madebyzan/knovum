pub(crate) mod concurrency;
mod decisions;
pub(crate) mod interactions;
pub(crate) mod intervention;
pub(crate) mod local_actions;
mod mcp;
pub(super) mod media;
mod receipts;
pub(crate) use mcp::{invoke, ToolInvocation, ToolInvocationOutcome};
use serde_json::Value;

pub(in crate::spaces) struct LocalActionOperationOutcome {
    pub id: String,
    pub output: Option<Value>,
    pub target_selection: Option<Value>,
}

fn arguments_digest(arguments: &Value) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(arguments).map_err(|error| error.to_string())?)
    ))
}

pub(crate) async fn require_reconciled(
    pool: &sqlx::SqlitePool,
    run_id: &str,
) -> Result<(), String> {
    let mut connection = pool.acquire().await.map_err(crate::spaces::db::sql_error)?;
    require_reconciled_on(&mut connection, run_id).await
}

pub(super) async fn require_reconciled_on(
    connection: &mut sqlx::SqliteConnection,
    run_id: &str,
) -> Result<(), String> {
    let pending: i64 = sqlx::query_scalar("SELECT count(*) FROM spaces_conversation_tool_operations WHERE run_id=$1 AND read_only=0 AND state='outcome_unknown' AND resolution IS NULL")
        .bind(run_id).fetch_one(connection).await.map_err(crate::spaces::db::sql_error)?;
    if pending != 0 {
        Err(
            "external-tool-outcome-unknown: review the original operation before further writes"
                .into(),
        )
    } else {
        Ok(())
    }
}

pub(crate) async fn recover(pool: &sqlx::SqlitePool) -> Result<(), String> {
    let dispatched: Vec<String> = sqlx::query_scalar("SELECT id FROM spaces_conversation_tool_operations WHERE state='dispatching' AND json_extract(source,'$.type')!='space_action'")
        .fetch_all(pool).await.map_err(crate::spaces::db::sql_error)?;
    for id in dispatched {
        receipts::complete(
            pool,
            &id,
            "outcome_unknown",
            &receipts::failure("mcp-process-ended-before-receipt-was-recorded"),
        )
        .await?;
    }
    sqlx::query("UPDATE spaces_runs SET state='waiting',wait_reason='tool_outcome_unknown',state_version=state_version+1 WHERE completed_at IS NULL AND cancel_requested_at IS NULL AND pause_requested_at IS NULL
        AND EXISTS(SELECT 1 FROM spaces_conversation_tool_operations o WHERE o.run_id=spaces_runs.id AND o.state='outcome_unknown' AND o.resolution IS NULL)")
        .execute(pool).await.map_err(crate::spaces::db::sql_error)?;
    sqlx::query("UPDATE spaces_conversation_mcp_interactions SET response='{\"action\":\"cancel\"}' WHERE response IS NULL").execute(pool).await.map_err(crate::spaces::db::sql_error)?;
    Ok(())
}

#[cfg(test)]
mod tests;
