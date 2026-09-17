use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageWorkflowRunRow {
    pub id: String,
    pub workflow_id: String,
    pub workflow_version: i64,
    pub status: String,
    pub error: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: i64,
    pub steps: Vec<PackageWorkflowStepRow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageWorkflowStepRow {
    pub id: String,
    pub action: String,
    pub status: String,
    pub output_node_ids: Vec<String>,
    pub error: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: Option<i64>,
}
