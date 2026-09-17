use serde::Deserialize;

use crate::ai::models::ExecutionTargetSelection;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GroundedKnowledgeRequest {
    pub job_id: String,
    pub space_id: String,
    pub scope: String,
    pub node_ids: Vec<String>,
    pub question: String,
    pub thread_id: Option<String>,
    pub target: ExecutionTargetSelection,
}
