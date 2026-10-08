use super::intervention::ToolOperationDecision;
use crate::spaces::db;
use sqlx::{Row, SqlitePool};

pub(super) async fn apply(
    pool: &SqlitePool,
    operation_id: &str,
    decision: ToolOperationDecision,
) -> Result<String, String> {
    let mut tx = db::begin_write(pool).await?;
    let row = sqlx::query("SELECT run_id,state,approval,resolution FROM spaces_conversation_tool_operations WHERE id=$1").bind(operation_id).fetch_one(&mut *tx).await.map_err(db::sql_error)?;
    let run_id: String = row.try_get("run_id").map_err(db::sql_error)?;
    let state: String = row.try_get("state").map_err(db::sql_error)?;
    let now = crate::clock::unix_time_millis()?;
    let active: Option<i64> = sqlx::query_scalar("SELECT 1 FROM spaces_runs WHERE id=$1 AND completed_at IS NULL AND cancel_requested_at IS NULL")
        .bind(&run_id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?;
    active.ok_or("conversation-tool-run-inactive")?;
    match decision {
        ToolOperationDecision::Approve | ToolOperationDecision::Deny => {
            if state != "prepared"
                || row
                    .try_get::<Option<String>, _>("approval")
                    .map_err(db::sql_error)?
                    .is_some()
            {
                return Err("tool-approval-stale".into());
            }
            let approved = matches!(decision, ToolOperationDecision::Approve);
            sqlx::query("UPDATE spaces_conversation_tool_operations SET approval=$1 WHERE id=$2 AND approval IS NULL")
                .bind(if approved { "allow" } else { "deny" }).bind(operation_id).execute(&mut *tx).await.map_err(db::sql_error)?;
            if !approved {
                super::receipts::complete_in_transaction(
                    &mut tx,
                    operation_id,
                    "failed",
                    &super::receipts::failure("User denied this external operation."),
                    now,
                )
                .await?;
            }
        }
        ToolOperationDecision::NewAttempt => {
            if state != "outcome_unknown"
                || row
                    .try_get::<Option<String>, _>("resolution")
                    .map_err(db::sql_error)?
                    .is_some()
            {
                return Err("tool-outcome-intervention-stale".into());
            }
            let budget: Option<i64> = sqlx::query_scalar("SELECT 1 FROM spaces_runs r WHERE id=$1 AND $2 < created_at+json_extract(budget,'$.maxDurationSeconds')*1000 AND (SELECT count(*) FROM spaces_conversation_tool_operations WHERE run_id=r.id) < json_extract(budget,'$.maxTools')")
                .bind(&run_id).bind(now).fetch_optional(&mut *tx).await.map_err(db::sql_error)?;
            budget.ok_or("conversation-tool-budget-exhausted")?;
            sqlx::query("UPDATE spaces_conversation_tool_operations SET resolution='new_attempt' WHERE id=$1").bind(operation_id).execute(&mut *tx).await.map_err(db::sql_error)?;
            sqlx::query("INSERT INTO spaces_conversation_tool_operations(id,run_id,task_id,graph_namespace,checkpoint_id,message_id,provider_call_id,wire_name,source,invocation_id,attempt,context,context_digest,read_only,approval,state,created_at)
                SELECT $1,run_id,task_id,graph_namespace,checkpoint_id,message_id,provider_call_id,wire_name,source,invocation_id,attempt+1,context,context_digest,read_only,'allow','prepared',$2 FROM spaces_conversation_tool_operations WHERE id=$3")
                .bind(uuid::Uuid::new_v4().to_string()).bind(now).bind(operation_id).execute(&mut *tx).await.map_err(db::sql_error)?;
        }
        ToolOperationDecision::Stop => {
            if state != "outcome_unknown"
                || row
                    .try_get::<Option<String>, _>("resolution")
                    .map_err(db::sql_error)?
                    .is_some()
            {
                return Err("tool-outcome-intervention-stale".into());
            }
            sqlx::query(
                "UPDATE spaces_conversation_tool_operations SET resolution='stop' WHERE id=$1",
            )
            .bind(operation_id)
            .execute(&mut *tx)
            .await
            .map_err(db::sql_error)?;
        }
    }
    tx.commit().await.map_err(db::sql_error)?;
    Ok(run_id)
}
