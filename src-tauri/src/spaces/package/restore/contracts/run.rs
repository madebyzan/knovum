use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageRun {
    pub id: String,
    pub parent_run_id: Option<String>,
    pub retry_of_run_id: Option<String>,
    pub fork_of_run_id: Option<String>,
    pub principal_kind: String,
    pub principal_id: String,
    pub kind: String,
    pub state: String,
    pub state_version: i64,
    pub policy_snapshot: Value,
    pub input_snapshot: Value,
    pub budget: Value,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub step_attempts: i64,
    pub expanded_steps: i64,
    pub output_bytes: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageRunStep {
    pub id: String,
    pub run_id: String,
    pub definition_node_id: String,
    pub instance_path: String,
    pub node_kind: String,
    pub depends_on: Value,
    pub configuration: Value,
    pub state: String,
    pub state_version: i64,
    pub resolved_input: Value,
    pub output: Option<Value>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageRunAttempt {
    pub id: String,
    pub run_id: String,
    pub step_instance_id: String,
    pub attempt_number: i64,
    pub state: String,
    pub request_digest: String,
    pub idempotency_key: String,
    pub external_task_id: Option<String>,
    pub checkpoint: Option<Value>,
    pub output: Option<Value>,
    pub error_code: Option<String>,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageRunEvent {
    pub run_id: String,
    pub step_instance_id: Option<String>,
    pub attempt_id: Option<String>,
    pub kind: crate::spaces::runs::RunEventKind,
    pub payload: Value,
    pub created_at: i64,
}
