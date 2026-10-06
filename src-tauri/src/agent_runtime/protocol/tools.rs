use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{control::AssistantExecutionIdentity, messages::AssistantContentPart};

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantToolDefinition {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) parameters: Value,
    pub(crate) read_only: bool,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantToolInvocation {
    pub(crate) identity: AssistantExecutionIdentity,
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
pub(crate) enum AssistantToolOutcome {
    Completed {
        value: Value,
        media: Vec<AssistantContentPart>,
    },
    AwaitChild {
        run_id: String,
    },
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantToolAdmission {
    pub(crate) sibling_tools: Vec<String>,
    pub(crate) identity: AssistantExecutionIdentity,
    pub(crate) graph_namespace: String,
    pub(crate) message_id: String,
    pub(crate) tool_call_id: String,
    pub(crate) name: String,
}
