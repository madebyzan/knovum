use serde::Serialize;
use tauri::AppHandle;

use super::{definition::WorkflowNode, WorkflowDefinition};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowReadinessIssue {
    pub code: &'static str,
    pub node_id: String,
    pub message: String,
}

pub async fn inspect(
    app: &AppHandle,
    definition: &WorkflowDefinition,
) -> Result<Vec<WorkflowReadinessIssue>, String> {
    let pool = crate::spaces::db::pool(app).await?;
    let mut issues = Vec::new();
    for node in &definition.nodes {
        match node {
            WorkflowNode::Action { id, action_id, .. } => {
                if let Some(found) = action(app, id, action_id) {
                    issues.push(found);
                }
            }
            WorkflowNode::DelegateTask { id, runtime_id } if runtime_id.trim().is_empty() => {
                issues.push(issue(
                    "runtime_unconfigured",
                    id,
                    "External Agent runtime is not selected",
                ));
            }
            WorkflowNode::ExternalTool {
                id,
                binding_id,
                schema_fingerprint,
            } => match super::external_tools::repository::load(pool, binding_id).await {
                Ok(binding) if !binding.enabled => issues.push(issue(
                    "external_tool_disabled",
                    id,
                    "External tool binding is disabled",
                )),
                Ok(binding) if binding.schema_fingerprint != *schema_fingerprint => {
                    issues.push(issue(
                        "external_tool_schema_changed",
                        id,
                        "External tool schema must be reviewed again",
                    ))
                }
                Err(error) => issues.push(issue("external_tool_unavailable", id, &error)),
                _ => {}
            },
            WorkflowNode::Subworkflow {
                id,
                workflow_id,
                version,
            } => {
                if let Err(error) =
                    super::repository::load_version(pool, workflow_id, i64::from(*version)).await
                {
                    issues.push(issue("subworkflow_unavailable", id, &error));
                }
            }
            _ => {}
        }
    }
    Ok(issues)
}

fn action(app: &AppHandle, node_id: &str, action_id: &str) -> Option<WorkflowReadinessIssue> {
    let definition = crate::spaces::actions::require(action_id).ok()?;
    let (Some(purpose), Some(capability)) =
        (definition.routing_purpose, definition.execution_capability)
    else {
        return None;
    };
    let result = crate::ai::settings::default_selection(app, purpose)
        .and_then(|selection| crate::ai::settings::resolve(app, selection, capability));
    result
        .err()
        .map(|error| issue("execution_target_unavailable", node_id, &error.to_string()))
}

fn issue(code: &'static str, node_id: &str, message: &str) -> WorkflowReadinessIssue {
    WorkflowReadinessIssue {
        code,
        node_id: node_id.to_owned(),
        message: message.to_owned(),
    }
}
