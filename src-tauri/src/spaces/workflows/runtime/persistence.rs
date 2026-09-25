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
        let state = if depends_on.is_empty() {
            "ready"
        } else {
            "pending"
        };
        sqlx::query(
            "INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,
             depends_on,configuration,state,state_version,resolved_input,created_at,updated_at)
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,0,$9,$10,$10)",
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
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
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
