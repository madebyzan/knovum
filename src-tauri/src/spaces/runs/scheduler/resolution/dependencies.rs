use sqlx::{Row, Sqlite, Transaction};

use super::{
    dependency_facts, dependency_input, join_policy, next_state, run_failure_policy,
    run_has_breakpoint,
};
use crate::spaces::{db, runs::RunEventKind};

pub(crate) async fn resolve_dependents(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    now: i64,
) -> Result<(), String> {
    loop {
        let rows = sqlx::query("SELECT id,definition_node_id,depends_on,node_kind,configuration FROM spaces_run_steps WHERE run_id=$1 AND state='pending'")
            .bind(run_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
        let mut changed = false;
        for row in rows {
            changed |= resolve_dependent(tx, run_id, &row, now).await?
        }
        if !changed {
            break;
        }
    }
    Ok(())
}

async fn resolve_dependent(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    row: &sqlx::sqlite::SqliteRow,
    now: i64,
) -> Result<bool, String> {
    let id: String = row.try_get("id").map_err(db::sql_error)?;
    let node_id: String = row.try_get("definition_node_id").map_err(db::sql_error)?;
    let encoded: String = row.try_get("depends_on").map_err(db::sql_error)?;
    let node_kind: String = row.try_get("node_kind").map_err(db::sql_error)?;
    let configuration: String = row.try_get("configuration").map_err(db::sql_error)?;
    let dependencies: Vec<String> = serde_json::from_str(&encoded).map_err(db::json_error)?;
    let facts = dependency_facts(tx, run_id, &dependencies).await?;
    let policy = run_failure_policy(tx, run_id).await?;
    let collect_results = node_kind == "join" && join_policy(&configuration)? == "collect_results";
    let Some(state) = next_state(
        &facts
            .iter()
            .map(|(state, _)| state.clone())
            .collect::<Vec<_>>(),
        collect_results || policy == "collect_errors",
    ) else {
        return Ok(false);
    };
    let input = (state == "ready")
        .then(|| dependency_input(&dependencies, &facts, collect_results))
        .transpose()?;
    let breakpoint = state == "ready" && run_has_breakpoint(tx, run_id, &node_id).await?;
    let next_state = if breakpoint { "paused" } else { state };
    let result = sqlx::query("UPDATE spaces_run_steps SET state=$1,resolved_input=COALESCE($2,resolved_input),wait_reason=$3,state_version=state_version+1,updated_at=$4 WHERE id=$5 AND state='pending'")
        .bind(next_state).bind(input).bind(breakpoint.then_some("debug_breakpoint"))
        .bind(now).bind(&id).execute(&mut **tx).await.map_err(db::sql_error)?;
    if result.rows_affected() == 1 && breakpoint {
        sqlx::query("UPDATE spaces_runs SET state='paused',wait_reason='debug_breakpoint',state_version=state_version+1,updated_at=$1 WHERE id=$2 AND completed_at IS NULL")
            .bind(now).bind(run_id).execute(&mut **tx).await.map_err(db::sql_error)?;
        super::super::super::append_event(
            tx,
            run_id,
            Some(&id),
            None,
            RunEventKind::DebugBreakpointReached,
            &serde_json::json!({"nodeId": node_id}),
            now,
        )
        .await?;
    }
    Ok(result.rows_affected() == 1)
}
