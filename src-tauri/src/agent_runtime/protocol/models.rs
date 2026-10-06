use serde::{Deserialize, Serialize};

use crate::ai::contracts::ChatModelProtocol;

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantModelBinding {
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
pub(crate) enum AssistantModelPurpose {
    Main,
    Research,
    ContextSummary,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantModelAdmission {
    pub(crate) identity: super::control::AssistantExecutionIdentity,
    pub(crate) request_id: String,
    pub(crate) purpose: AssistantModelPurpose,
    pub(crate) checkpoint_id: Option<String>,
    pub(crate) estimated_input_tokens: u64,
    pub(crate) graph_namespace: String,
    pub(crate) max_output_tokens: u32,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantModelCompletion {
    pub(crate) identity: super::control::AssistantExecutionIdentity,
    pub(crate) request_id: String,
    pub(crate) input_tokens: Option<u64>,
    pub(crate) output_tokens: Option<u64>,
    pub(crate) message_id: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantModelFailure {
    pub(crate) identity: super::control::AssistantExecutionIdentity,
    pub(crate) request_id: String,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantModelCheckpoint {
    pub(crate) graph_namespace: String,
    pub(crate) message_ids: Vec<String>,
    pub(crate) request_ids: Vec<String>,
}
