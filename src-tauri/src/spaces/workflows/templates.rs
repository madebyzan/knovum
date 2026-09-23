use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use ts_rs::TS;

use super::{repository, WorkflowDefinition, WorkflowDraft, WorkflowVersion};

mod recipes;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Deserialize, TS)]
#[ts(export_to = "BuiltInWorkflowTemplateId.ts")]
#[serde(rename_all = "kebab-case")]
pub(crate) enum BuiltInWorkflowTemplateId {
    PresentationProduction,
    VideoProduction,
    SocialContentPackage,
    SubtitledVideoProduction,
    MeetingContentPackage,
}

#[derive(Deserialize, TS)]
#[ts(export_to = "InstantiateWorkflowTemplateRequest.ts")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct InstantiateWorkflowTemplateRequest {
    space_id: String,
    template_id: BuiltInWorkflowTemplateId,
    parameters: Value,
}

pub(crate) async fn instantiate(
    pool: &SqlitePool,
    request: InstantiateWorkflowTemplateRequest,
) -> Result<WorkflowVersion, String> {
    let definition = definition(request.template_id, &request.parameters)?;
    let workflow_id = uuid::Uuid::new_v4().to_string();
    repository::save_draft(
        pool,
        WorkflowDraft {
            id: workflow_id.clone(),
            space_id: request.space_id,
            revision: 0,
            definition,
            created_at: 0,
            updated_at: 0,
        },
    )
    .await?;
    repository::publish(pool, &workflow_id, 0).await
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
