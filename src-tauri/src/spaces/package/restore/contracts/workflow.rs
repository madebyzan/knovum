use serde::Deserialize;

use crate::spaces::workflows::WorkflowDefinition;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageWorkflowDraftRow {
    pub id: String,
    pub revision: i64,
    pub definition: WorkflowDefinition,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageWorkflowVersionRow {
    pub workflow_id: String,
    pub version: i64,
    pub definition: WorkflowDefinition,
    pub published_at: i64,
}
