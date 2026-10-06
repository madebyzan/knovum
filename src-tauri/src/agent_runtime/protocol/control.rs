use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    messages::AssistantInputMessage, models::AssistantModelBinding, tools::AssistantToolDefinition,
};

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantInitialize {
    pub(crate) protocol_version: u16,
    pub(crate) graph_version: String,
    pub(crate) checkpoint_path: String,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantReady {
    pub(crate) protocol_version: u16,
    pub(crate) graph_version: String,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantExecutionIdentity {
    pub(crate) run_id: String,
    pub(crate) thread_id: String,
    pub(crate) execution_revision: String,
    pub(crate) control_epoch: u64,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantExecute {
    pub(crate) identity: AssistantExecutionIdentity,
    pub(crate) model: AssistantModelBinding,
    pub(crate) system_prompt: String,
    pub(crate) tools: Vec<AssistantToolDefinition>,
    pub(crate) messages: Vec<AssistantInputMessage>,
    pub(crate) input_id: String,
    pub(crate) after_checkpoint_id: Option<String>,
    pub(crate) resume: Option<Value>,
    pub(crate) skills: std::collections::BTreeMap<String, String>,
    pub(crate) memory: std::collections::BTreeMap<String, String>,
    pub(crate) plan: bool,
    pub(crate) research: bool,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantControl {
    pub(crate) identity: AssistantExecutionIdentity,
    pub(crate) operation: AssistantControlOperation,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AssistantControlOperation {
    Pause,
    Cancel,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantExecutionResult {
    pub(crate) identity: AssistantExecutionIdentity,
    pub(crate) checkpoint_id: String,
    pub(crate) state: AssistantExecutionState,
    pub(crate) interrupts: Vec<Value>,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AssistantExecutionState {
    Completed,
    Interrupted,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantPruneThreads {
    pub(crate) thread_ids: Vec<String>,
    pub(crate) retained_thread_ids: Vec<String>,
    pub(crate) orphan_before_ms: i64,
}
