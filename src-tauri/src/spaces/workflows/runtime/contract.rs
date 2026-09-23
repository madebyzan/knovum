use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StartWorkflowRequest {
    pub space_id: String,
    pub workflow_id: String,
    pub version: i64,
    pub input: Value,
    pub parent_run_id: Option<String>,
    #[serde(default)]
    pub grant_id: Option<String>,
    #[serde(default)]
    pub test_mode: bool,
    #[serde(default)]
    pub breakpoint_node_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StartWorkflowStepTestRequest {
    pub space_id: String,
    pub workflow_id: String,
    pub version: i64,
    pub node_id: String,
    pub input: Value,
}
