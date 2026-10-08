use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{messages::AgentInputMessage, models::AgentModelBinding, tools::AgentToolDefinition};

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentInitialize {
    pub(crate) protocol_version: u16,
    pub(crate) graph_version: String,
    pub(crate) checkpoint_path: String,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentReady {
    pub(crate) protocol_version: u16,
    pub(crate) graph_version: String,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentExecutionIdentity {
    pub(crate) run_id: String,
    pub(crate) thread_id: String,
    pub(crate) execution_revision: String,
    pub(crate) control_epoch: u64,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentExecute {
    pub(crate) identity: AgentExecutionIdentity,
    pub(crate) model: AgentModelBinding,
    pub(crate) system_prompt: String,
    pub(crate) tools: Vec<AgentToolDefinition>,
    pub(crate) messages: Vec<AgentInputMessage>,
    pub(crate) input_id: String,
    pub(crate) after_checkpoint_id: Option<String>,
    pub(crate) resume: Option<Value>,
    pub(crate) skills: Vec<crate::extensions::contracts::AgentSkillFile>,
    pub(crate) skill_tool_bindings: std::collections::BTreeMap<String, Vec<String>>,
    pub(crate) memory: std::collections::BTreeMap<String, String>,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentControl {
    pub(crate) identity: AgentExecutionIdentity,
    pub(crate) operation: AgentControlOperation,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AgentControlOperation {
    Pause,
    Cancel,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentExecutionObservation {
    pub(crate) identity: AgentExecutionIdentity,
    pub(crate) checkpoint_id: String,
    pub(crate) state: AgentExecutionState,
    pub(crate) interrupts: Vec<Value>,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AgentExecutionState {
    Completed,
    Interrupted,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentPruneThreads {
    pub(crate) thread_ids: Vec<String>,
    pub(crate) retained_thread_ids: Vec<String>,
    pub(crate) orphan_before_ms: i64,
}
