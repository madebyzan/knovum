use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize, JsonSchema)]
pub(super) struct SearchArgs {
    pub(super) space_id: String,
    pub(super) query: String,
    #[serde(default = "default_limit")]
    pub(super) limit: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(super) struct NodeArgs {
    pub(super) space_id: String,
    pub(super) node_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(super) struct ProvenanceArgs {
    pub(super) space_id: String,
    pub(super) target_kind: String,
    pub(super) target_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(super) struct HistoryArgs {
    pub(super) space_id: String,
    #[serde(default)]
    pub(super) entity_kind: Option<String>,
    #[serde(default)]
    pub(super) entity_id: Option<String>,
    #[serde(default = "default_limit")]
    pub(super) limit: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(super) struct PathArgs {
    pub(super) space_id: String,
    pub(super) source_id: String,
    pub(super) target_id: String,
    #[serde(default = "default_depth")]
    pub(super) max_depth: usize,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(super) struct CommitKnowledgeArgs {
    pub(super) space_id: String,
    pub(super) idempotency_key: String,
    pub(super) operations: Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(super) struct SpaceArgs {
    pub(super) space_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(super) struct RunArgs {
    pub(super) space_id: String,
    pub(super) run_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(super) struct RunEventsArgs {
    pub(super) space_id: String,
    pub(super) run_id: String,
    #[serde(default)]
    pub(super) after_sequence: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(super) struct RetryRunArgs {
    pub(super) space_id: String,
    pub(super) run_id: String,
    pub(super) step_ids: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(super) struct InvokeActionArgs {
    pub(super) space_id: String,
    pub(super) action_id: String,
    pub(super) action_version: u16,
    pub(super) idempotency_key: String,
    pub(super) parameters: Value,
    #[serde(default)]
    pub(super) input_snapshot: Value,
    #[serde(default)]
    pub(super) expected_revisions: Value,
}

fn default_limit() -> i64 {
    30
}
fn default_depth() -> usize {
    6
}
