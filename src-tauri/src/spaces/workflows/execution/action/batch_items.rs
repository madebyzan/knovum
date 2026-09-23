use serde_json::Value;
use sqlx::SqlitePool;

use crate::spaces::{actions::ActionInvocationOutcome, db, runs::AttemptClaim};

mod state;

use state::{lineage, node_id, reusable};

pub(super) async fn prepare(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    item_key: &str,
    input: &Value,
) -> Result<Option<ActionInvocationOutcome>, String> {
    let lineage = lineage(claim);
    let existing = sqlx::query(
        "SELECT state,output,child_run_id FROM spaces_run_batch_items
         WHERE lineage_run_id=$1 AND definition_node_id=$2 AND item_key=$3",
    )
    .bind(lineage)
    .bind(node_id(claim))
    .bind(item_key)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    if let Some(row) = existing.as_ref() {
        if let Some(outcome) = reusable(pool, row).await? {
            return Ok(Some(outcome));
        }
    }
    if existing.is_none() {
        crate::spaces::runs::reserve_expanded_steps(pool, &claim.run_id, 1).await?;
    }
    let now = crate::clock::unix_time_millis()?;
    sqlx::query(
        "INSERT INTO spaces_run_batch_items(lineage_run_id,definition_node_id,item_key,
         latest_run_id,latest_step_instance_id,state,attempt_count,input,updated_at)
         VALUES($1,$2,$3,$4,$5,'running',1,$6,$7)
         ON CONFLICT(lineage_run_id,definition_node_id,item_key) DO UPDATE SET
         latest_run_id=excluded.latest_run_id,latest_step_instance_id=excluded.latest_step_instance_id,
         state='running',attempt_count=attempt_count+1,input=excluded.input,output=NULL,error=NULL,
         child_run_id=NULL,updated_at=excluded.updated_at",
    )
    .bind(lineage)
    .bind(node_id(claim))
    .bind(item_key)
    .bind(&claim.run_id)
    .bind(&claim.step_instance_id)
    .bind(serde_json::to_string(input).map_err(db::json_error)?)
    .bind(now)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(None)
}

pub(super) async fn finish(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    item_key: &str,
    outcome: &Result<ActionInvocationOutcome, String>,
) -> Result<(), String> {
    let (state, output, error, child_run_id) = match outcome {
        Ok(ActionInvocationOutcome::QueryResult { result }) => {
            ("succeeded", Some(result), None, None)
        }
        Ok(ActionInvocationOutcome::CommittedReceipt { receipt }) => {
            ("succeeded", Some(receipt), None, None)
        }
        Ok(ActionInvocationOutcome::AcceptedRun { run_id }) => {
            ("waiting", None, None, Some(run_id.as_str()))
        }
        Ok(ActionInvocationOutcome::RequiresApproval { .. }) => ("waiting", None, None, None),
        Err(error) => ("failed", None, Some(error.as_str()), None),
    };
    sqlx::query(
        "UPDATE spaces_run_batch_items SET state=$1,output=$2,error=$3,child_run_id=$4,updated_at=$5
         WHERE lineage_run_id=$6 AND definition_node_id=$7 AND item_key=$8",
    )
    .bind(state)
    .bind(output.map(serde_json::to_string).transpose().map_err(db::json_error)?)
    .bind(error)
    .bind(child_run_id)
    .bind(crate::clock::unix_time_millis()?)
    .bind(lineage(claim))
    .bind(node_id(claim))
    .bind(item_key)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub(in crate::spaces::workflows::execution) async fn prepare_child(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    item_key: &str,
    input: &Value,
) -> Result<Option<String>, String> {
    let child: Option<String> = sqlx::query_scalar(
        "SELECT child_run_id FROM spaces_run_batch_items
         WHERE lineage_run_id=$1 AND definition_node_id=$2 AND item_key=$3",
    )
    .bind(lineage(claim))
    .bind(node_id(claim))
    .bind(item_key)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .flatten();
    if let Some(child_id) = child {
        let state: Option<String> = sqlx::query_scalar("SELECT state FROM spaces_runs WHERE id=$1")
            .bind(&child_id)
            .fetch_optional(pool)
            .await
            .map_err(db::sql_error)?;
        if state.as_deref().is_some_and(|state| {
            matches!(
                state,
                "queued" | "running" | "waiting" | "paused" | "succeeded" | "completed_with_errors"
            )
        }) {
            return Ok(Some(child_id));
        }
    }
    prepare(pool, claim, item_key, input).await?;
    Ok(None)
}

pub(in crate::spaces::workflows::execution) async fn attach_child(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    item_key: &str,
    child_run_id: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE spaces_run_batch_items SET state='waiting',child_run_id=$1,updated_at=$2
         WHERE lineage_run_id=$3 AND definition_node_id=$4 AND item_key=$5",
    )
    .bind(child_run_id)
    .bind(crate::clock::unix_time_millis()?)
    .bind(lineage(claim))
    .bind(node_id(claim))
    .bind(item_key)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}
