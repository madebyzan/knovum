use std::collections::HashMap;

use serde_json::{Map, Value};
use sqlx::{Row, SqlitePool};

use super::definition::{
    FailurePolicy, WorkflowDefinition, WorkflowDraft, WorkflowEdge, WorkflowLimits, WorkflowNode,
    WorkflowNodeLayout, WorkflowPort, WorkflowValueType,
};
use crate::spaces::db;

pub(super) async fn extract(
    pool: &SqlitePool,
    space_id: &str,
    title: &str,
) -> Result<WorkflowDraft, String> {
    if space_id.trim().is_empty() || title.trim().is_empty() {
        return Err("workflow-history-invalid: Space and title are required".into());
    }
    let rows = sqlx::query(
        "SELECT action_id,action_version FROM spaces_action_invocations
         WHERE space_id=$1 AND outcome_kind!='requires_approval'
         ORDER BY created_at DESC LIMIT 20",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    if rows.is_empty() {
        return Err("workflow-history-empty: no completed Space Actions are available".into());
    }
    let mut actions = rows
        .into_iter()
        .rev()
        .map(action_node)
        .collect::<Result<Vec<_>, _>>()?;
    let mut nodes = vec![WorkflowNode::Input {
        id: "input".into(),
        output_type: WorkflowValueType::Value,
    }];
    nodes.append(&mut actions);
    nodes.push(WorkflowNode::Output {
        id: "output".into(),
        input_type: WorkflowValueType::Value,
    });
    let ids = nodes
        .iter()
        .map(|node| node.id().to_owned())
        .collect::<Vec<_>>();
    let edges = ids
        .windows(2)
        .enumerate()
        .map(|(index, pair)| WorkflowEdge {
            id: format!("history-edge-{}", index + 1),
            source_node_id: pair[0].clone(),
            source_port: WorkflowPort::Output,
            target_node_id: pair[1].clone(),
            target_port: WorkflowPort::Input,
            branch: None,
        })
        .collect();
    let layout = ids
        .into_iter()
        .enumerate()
        .map(|(index, id)| {
            let column = u32::try_from(index)
                .map_err(|_| "workflow-history-invalid: layout index overflow")?;
            Ok((
                id,
                WorkflowNodeLayout {
                    x: 80.0 + f64::from(column) * 180.0,
                    y: 100.0,
                },
            ))
        })
        .collect::<Result<HashMap<_, _>, String>>()?;
    Ok(WorkflowDraft {
        id: uuid::Uuid::new_v4().to_string(),
        space_id: space_id.to_owned(),
        revision: 0,
        definition: WorkflowDefinition {
            schema_version: 1,
            title: title.trim().to_owned(),
            nodes,
            edges,
            layout,
            failure_policy: FailurePolicy::StopRun,
            limits: WorkflowLimits {
                max_expanded_steps: 10_000,
                max_parallelism: 4,
                max_duration_seconds: 3_600,
                max_total_tokens: 200_000,
                max_step_attempts: 100,
                max_input_bytes: 16_777_216,
                max_output_bytes: 67_108_864,
            },
        },
        created_at: 0,
        updated_at: 0,
    })
}

fn action_node(row: sqlx::sqlite::SqliteRow) -> Result<WorkflowNode, String> {
    let action_id: String = row.try_get("action_id").map_err(db::sql_error)?;
    let action_version: u16 = row.try_get("action_version").map_err(db::sql_error)?;
    let definition = crate::spaces::actions::require(&action_id)?;
    let schema = crate::spaces::actions::parameter_schema(definition);
    let parameters = schema
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|key| (key.to_owned(), Value::Null))
        .collect::<Map<_, _>>();
    Ok(WorkflowNode::Action {
        id: format!("{}-{}", action_id, uuid::Uuid::new_v4().simple()),
        action_id,
        action_version,
        parameters: Value::Object(parameters),
    })
}
