use std::collections::HashMap;

use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use crate::spaces::db;

use super::super::definition::{WorkflowEdge, WorkflowNode};

pub(super) struct StepPlan<'a> {
    pub nodes: &'a [WorkflowNode],
    pub edges: &'a [WorkflowEdge],
    pub order: &'a [String],
    pub input: &'a Value,
    pub breakpoint_node_ids: &'a [String],
}

pub(super) fn validate_input_budget(input: &Value, max_bytes: u64) -> Result<(), String> {
    let bytes = serde_json::to_vec(input).map_err(db::json_error)?.len() as u64;
    if bytes > max_bytes {
        return Err("budget-exhausted: Workflow input exceeds maxInputBytes".into());
    }
    Ok(())
}

pub(super) async fn insert_steps(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    plan: StepPlan<'_>,
    now: i64,
) -> Result<(), String> {
    let dependencies = dependencies(plan.edges);
    let by_id = plan
        .nodes
        .iter()
        .map(|node| (node.id(), node))
        .collect::<HashMap<_, _>>();
    for (index, node_id) in plan.order.iter().enumerate() {
        let node = by_id[node_id.as_str()];
        let depends_on = dependencies
            .get(node_id.as_str())
            .cloned()
            .unwrap_or_default();
        let breakpoint = plan
            .breakpoint_node_ids
            .iter()
            .any(|candidate| candidate == node_id);
        let state = match (depends_on.is_empty(), breakpoint) {
            (true, true) => "paused",
            (true, false) => "ready",
            (false, _) => "pending",
        };
        sqlx::query(
            "INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,
             depends_on,configuration,state,state_version,resolved_input,wait_reason,created_at,updated_at)
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,0,$9,$10,$11,$11)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(run_id)
        .bind(node_id)
        .bind(format!("{index}:{node_id}"))
        .bind(node.kind())
        .bind(serde_json::to_string(&depends_on).map_err(db::json_error)?)
        .bind(serde_json::to_string(node).map_err(db::json_error)?)
        .bind(state)
        .bind(serde_json::to_string(plan.input).map_err(db::json_error)?)
        .bind(breakpoint.then_some("debug_breakpoint"))
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    Ok(())
}

pub(super) async fn pause_at_initial_breakpoint(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    now: i64,
) -> Result<(), String> {
    let steps = sqlx::query_scalar::<_, String>(
        "SELECT id FROM spaces_run_steps WHERE run_id=$1 AND state='paused'
         AND wait_reason='debug_breakpoint'",
    )
    .bind(run_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if steps.is_empty() {
        return Ok(());
    }
    sqlx::query(
        "UPDATE spaces_runs SET state='paused',wait_reason='debug_breakpoint',
         state_version=state_version+1,updated_at=$1 WHERE id=$2",
    )
    .bind(now)
    .bind(run_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for step_id in steps {
        crate::spaces::runs::append_event(
            tx,
            run_id,
            Some(&step_id),
            None,
            crate::spaces::runs::RunEventKind::DebugBreakpointReached,
            &serde_json::json!({}),
            now,
        )
        .await?;
    }
    Ok(())
}

fn dependencies(edges: &[WorkflowEdge]) -> HashMap<&str, Vec<String>> {
    let mut dependencies = HashMap::<&str, Vec<String>>::new();
    for edge in edges {
        dependencies
            .entry(edge.target_node_id.as_str())
            .or_default()
            .push(edge.source_node_id.clone());
    }
    dependencies
}
