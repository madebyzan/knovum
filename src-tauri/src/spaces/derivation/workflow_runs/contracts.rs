use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowStepStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

impl WorkflowStepStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveWorkflowRunRequest {
    pub id: String,
    pub space_id: String,
    pub workflow_id: String,
    pub workflow_version: i64,
    pub steps: Vec<SaveWorkflowStep>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveWorkflowStep {
    pub id: String,
    pub action: String,
    pub status: WorkflowStepStatus,
    pub output_node_ids: Vec<String>,
    #[serde(default)]
    pub error: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRunRecord {
    pub id: String,
    pub space_id: String,
    pub workflow_id: String,
    pub workflow_version: i64,
    pub status: String,
    pub error: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: Option<i64>,
    pub steps: Vec<WorkflowStepRecord>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStepRecord {
    pub id: String,
    pub action: String,
    pub status: String,
    pub output_node_ids: Vec<String>,
    pub error: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: Option<i64>,
}
