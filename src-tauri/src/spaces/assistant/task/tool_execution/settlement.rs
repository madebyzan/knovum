use serde_json::{json, Value};
use sqlx::SqlitePool;

use super::super::identity::ensure_active;
use crate::spaces::{actions::ActionInvocationOutcome, db, runs::RunState};

pub(super) async fn settle(
    pool: &SqlitePool,
    parent_run_id: &str,
    outcome: ActionInvocationOutcome,
) -> Result<Value, String> {
    match outcome {
        ActionInvocationOutcome::AcceptedRun { ref run_id } => {
            set_execution_state(pool, parent_run_id, "waiting", Some("executor")).await?;
            let value = await_child(pool, parent_run_id, run_id).await?;
            set_execution_state(pool, parent_run_id, "running", None).await?;
            Ok(value)
        }
        _ => serde_json::to_value(outcome).map_err(db::json_error),
    }
}

async fn await_child(
    pool: &SqlitePool,
    parent_run_id: &str,
    child_run_id: &str,
) -> Result<Value, String> {
    let mut changes = crate::spaces::runs::subscribe_to_changes();
    loop {
        ensure_active(pool, parent_run_id).await?;
        let run = crate::spaces::runs::load_run(pool, child_run_id).await?;
        if run.state.is_terminal() {
            if !matches!(
                run.state,
                RunState::Succeeded | RunState::CompletedWithErrors
            ) {
                return Err(format!("assistant-child-run-{}", state_text(run.state)));
            }
            let output = sqlx::query_scalar::<_, Option<String>>(
                "SELECT output FROM spaces_run_steps WHERE run_id=$1 AND state='succeeded' ORDER BY updated_at DESC LIMIT 1",
            ).bind(child_run_id).fetch_optional(pool).await.map_err(db::sql_error)?.flatten()
                .map(|encoded| serde_json::from_str(&encoded).map_err(db::json_error)).transpose()?
                .unwrap_or(Value::Null);
            let node_id: Option<String> = sqlx::query_scalar(
                "SELECT output_node_id FROM spaces_knowledge_derivations WHERE id=$1 AND status='completed'",
            )
            .bind(child_run_id)
            .fetch_optional(pool)
            .await
            .map_err(db::sql_error)?;
            return Ok(json!({"type": "accepted_run", "runId": child_run_id,
                "state": state_text(run.state), "output": output, "nodeId": node_id}));
        }
        changes
            .changed()
            .await
            .map_err(|_| "run-change-signal-closed".to_owned())?;
    }
}

async fn set_execution_state(
    pool: &SqlitePool,
    run_id: &str,
    state: &str,
    wait_reason: Option<&str>,
) -> Result<(), String> {
    let changed = sqlx::query(
        "UPDATE spaces_runs SET state=$1,wait_reason=$2,state_version=state_version+1,updated_at=$3
         WHERE id=$4 AND state IN ('running','waiting') AND cancel_requested_at IS NULL
         AND pause_requested_at IS NULL AND completed_at IS NULL",
    )
    .bind(state)
    .bind(wait_reason)
    .bind(crate::clock::unix_time_millis()?)
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        changed,
        "assistant-task-inactive: task was cancelled, paused, or completed",
    )?;
    crate::spaces::runs::announce_change();
    Ok(())
}

fn state_text(state: RunState) -> &'static str {
    match state {
        RunState::Queued => "queued",
        RunState::Running => "running",
        RunState::Waiting => "waiting",
        RunState::Paused => "paused",
        RunState::Succeeded => "succeeded",
        RunState::CompletedWithErrors => "completed_with_errors",
        RunState::Failed => "failed",
        RunState::Cancelled => "cancelled",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn accepted_child_settles_while_its_assistant_parent_is_waiting() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        db::migrate(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_runs(id,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('parent','space','assistant','principal','assistant','running','{}','{}','{}',1,1)").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_runs(id,home_space_id,parent_run_id,principal_kind,principal_id,kind,state,completed_at,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('child','space','parent','assistant','principal','action','succeeded',2,'{}','{}','{}',1,2)").execute(&pool).await.unwrap();
        let result = settle(
            &pool,
            "parent",
            ActionInvocationOutcome::AcceptedRun {
                run_id: "child".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(result["state"], "succeeded");
        sqlx::query("UPDATE spaces_runs SET cancel_requested_at=3,execution_epoch=execution_epoch+1 WHERE id='parent'")
            .execute(&pool).await.unwrap();
        assert!(set_execution_state(&pool, "parent", "running", None)
            .await
            .is_err());
        assert_eq!(
            crate::spaces::runs::load_run(&pool, "parent")
                .await
                .unwrap()
                .state,
            RunState::Running
        );
    }
}
