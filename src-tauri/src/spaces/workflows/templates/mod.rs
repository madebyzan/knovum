use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use ts_rs::TS;

use super::{definition::WorkflowNode, WorkflowDefinition};
use crate::ai::models::ExecutionTargetSelection;
use crate::spaces::runs::RunObservation;

mod recipes;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Deserialize, Serialize, TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/BuiltInWorkflowTemplateId.ts")]
#[serde(rename_all = "kebab-case")]
pub(crate) enum BuiltInWorkflowTemplateId {
    PresentationProduction,
    VideoProduction,
    SocialContentPackage,
    SubtitledVideoProduction,
    MeetingContentPackage,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct StartBuiltInWorkflowRequest {
    pub request_id: String,
    pub space_id: String,
    pub template_id: BuiltInWorkflowTemplateId,
    pub parameters: Value,
    pub input_node_ids: Vec<String>,
    pub targets: HashMap<String, ExecutionTargetSelection>,
}

pub(crate) async fn start(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    request: StartBuiltInWorkflowRequest,
) -> Result<RunObservation, String> {
    let definition = definition(request.template_id, &request.parameters)?;
    let mut required_targets = HashSet::new();
    for node in &definition.nodes {
        let WorkflowNode::Action { action_id, .. } = node else {
            continue;
        };
        let action = crate::spaces::actions::require(action_id)?;
        if let Some(capability) = action.execution_capability {
            required_targets.insert(action_id.as_str());
            let target = request
                .targets
                .get(action_id)
                .ok_or("workflow-target-missing: select a target for each model Action")?;
            crate::ai::settings::resolve(app, target.clone(), capability)
                .map_err(|error| format!("workflow-target-unavailable: {error}"))?;
        }
    }
    if request
        .targets
        .keys()
        .any(|id| !required_targets.contains(id.as_str()))
    {
        return Err("workflow-target-invalid: target does not belong to this recipe".into());
    }
    let input = json!({"inputNodeIds": request.input_node_ids, "targets": request.targets});
    let template_id =
        serde_json::to_value(request.template_id).map_err(crate::spaces::db::json_error)?;
    super::runtime::start(
        pool,
        &request.space_id,
        &request.request_id,
        template_id
            .as_str()
            .ok_or("workflow-template-invalid: template identity is invalid")?,
        definition,
        input,
    )
    .await
}

fn definition(
    template_id: BuiltInWorkflowTemplateId,
    parameters: &Value,
) -> Result<WorkflowDefinition, String> {
    match template_id {
        BuiltInWorkflowTemplateId::PresentationProduction => {
            recipes::reviewed_pair(recipes::ReviewedPair {
                title: "Presentation production",
                draft: ("document", "draft-document", take(parameters, "document")?),
                review_purpose: "Review the editable document before rendering",
                output: ("presentation", "render-presentation", json!({})),
            })
        }
        BuiltInWorkflowTemplateId::VideoProduction => {
            recipes::reviewed_pair(recipes::ReviewedPair {
                title: "Video production",
                draft: ("script", "draft-script", take(parameters, "script")?),
                review_purpose: "Review the editable script before video generation",
                output: ("video", "generate-video", take(parameters, "video")?),
            })
        }
        BuiltInWorkflowTemplateId::SocialContentPackage => recipes::social(parameters),
        BuiltInWorkflowTemplateId::SubtitledVideoProduction => recipes::subtitled_video(),
        BuiltInWorkflowTemplateId::MeetingContentPackage => recipes::meeting(parameters),
    }
}

fn take(parameters: &Value, key: &str) -> Result<Value, String> {
    parameters
        .get(key)
        .cloned()
        .ok_or_else(|| format!("workflow-template-invalid: {key} parameters are required"))
}
