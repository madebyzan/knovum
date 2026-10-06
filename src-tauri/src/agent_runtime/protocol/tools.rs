use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{control::AgentExecutionIdentity, messages::AgentContentPart};

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentToolDefinition {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) parameters: Value,
    pub(crate) read_only: bool,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentToolInvocation {
    pub(crate) identity: AgentExecutionIdentity,
    pub(crate) graph_namespace: String,
    pub(crate) checkpoint_id: String,
    pub(crate) message_id: String,
    pub(crate) tool_call_id: String,
    pub(crate) name: String,
    pub(crate) arguments: Value,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum AgentToolOutcome {
    AwaitUser {
        operation_id: String,
        reason: String,
    },
    Completed {
        value: Value,
        media: Vec<AgentContentPart>,
    },
    AwaitChild {
        run_id: String,
    },
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentToolAdmission {
    pub(crate) sibling_tools: Vec<String>,
    pub(crate) identity: AgentExecutionIdentity,
    pub(crate) graph_namespace: String,
    pub(crate) message_id: String,
    pub(crate) tool_call_id: String,
    pub(crate) name: String,
}
