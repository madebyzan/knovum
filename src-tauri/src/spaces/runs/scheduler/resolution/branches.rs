use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use crate::spaces::{db, runs::AttemptClaim};

pub(crate) async fn skip_unselected_branches(
    tx: &mut Transaction<'_, Sqlite>,
    claim: &AttemptClaim,
    selected: &str,
    now: i64,
) -> Result<(), String> {
    let definition = claim
        .policy_snapshot
        .get("definition")
        .ok_or("workflow-plan-invalid: definition snapshot is missing")?;
    let edges = definition
        .get("edges")
        .and_then(Value::as_array)
        .ok_or("workflow-plan-invalid: edges are missing")?;
    let node_id = sqlx::query_scalar::<_, String>(
        "SELECT definition_node_id FROM spaces_run_steps WHERE id=$1",
    )
    .bind(&claim.step_instance_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let nodes = definition
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or("workflow-plan-invalid: nodes are missing")?;
    for edge in edges
        .iter()
        .filter(|edge| edge.get("sourceNodeId").and_then(Value::as_str) == Some(&node_id))
    {
        let branch = edge.get("branch").and_then(Value::as_str);
        if branch.is_some() && branch != Some(selected) {
            if let Some(target) = edge.get("targetNodeId").and_then(Value::as_str) {
                skip_path(tx, &claim.run_id, target, edges, nodes, now).await?;
            }
        }
    }
    Ok(())
}

async fn skip_path(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    start: &str,
    edges: &[Value],
    nodes: &[Value],
    now: i64,
) -> Result<(), String> {
    let mut pending = vec![start.to_owned()];
    let mut visited = std::collections::HashSet::new();
    while let Some(node_id) = pending.pop() {
        if !visited.insert(node_id.clone()) {
            continue;
        }
        let kind = nodes
            .iter()
            .find(|node| node.get("id").and_then(Value::as_str) == Some(&node_id))
            .and_then(|node| node.get("kind"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        if matches!(kind, "join" | "output") {
            continue;
        }
        let changed = sqlx::query("UPDATE spaces_run_steps SET state='skipped',state_version=state_version+1,updated_at=$1 WHERE run_id=$2 AND definition_node_id=$3 AND state='pending'")
            .bind(now).bind(run_id).bind(&node_id).execute(&mut **tx).await.map_err(db::sql_error)?
            .rows_affected() == 1;
        if !changed {
            continue;
        }
        pending.extend(
            edges
                .iter()
                .filter(|edge| edge.get("sourceNodeId").and_then(Value::as_str) == Some(&node_id))
                .filter_map(|edge| {
                    edge.get("targetNodeId")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                }),
        );
    }
    Ok(())
}
