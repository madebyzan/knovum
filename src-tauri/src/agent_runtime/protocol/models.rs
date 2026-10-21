use serde::{Deserialize, Serialize};

use crate::ai::contracts::ChatModelProtocol;

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentModelBinding {
    pub(crate) protocol: ChatModelProtocol,
    pub(crate) model: String,
    pub(crate) base_url: String,
    pub(crate) api_version: Option<String>,
    pub(crate) api_key: Option<String>,
    pub(crate) temperature: Option<f64>,
    pub(crate) max_output_tokens: u32,
    pub(crate) max_input_tokens: u32,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AgentModelPurpose {
    Main,
    Research,
    ContextSummary,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentModelAdmission {
    pub(crate) identity: super::control::AgentExecutionIdentity,
    pub(crate) request_id: String,
    pub(crate) purpose: AgentModelPurpose,
    pub(crate) checkpoint_id: Option<String>,
    pub(crate) graph_namespace: String,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentModelCompletion {
    pub(crate) identity: super::control::AgentExecutionIdentity,
    pub(crate) request_id: String,
    pub(crate) input_tokens: Option<u64>,
    pub(crate) output_tokens: Option<u64>,
    pub(crate) message_id: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentModelFailure {
    pub(crate) identity: super::control::AgentExecutionIdentity,
    pub(crate) request_id: String,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentModelCheckpoint {
    pub(crate) graph_namespace: String,
    pub(crate) message_ids: Vec<String>,
    pub(crate) request_ids: Vec<String>,
}
