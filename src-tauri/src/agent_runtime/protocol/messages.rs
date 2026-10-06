use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum AgentContentPart {
    Text { text: String },
    Image { mime_type: String, data: String },
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentInputMessage {
    pub(crate) id: String,
    pub(crate) role: AgentInputRole,
    pub(crate) content: Vec<AgentContentPart>,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AgentInputRole {
    User,
    Assistant,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentProjectionRequest {
    pub(crate) thread_id: String,
    pub(crate) after_checkpoint_id: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentProjectedMessage {
    pub(crate) message_id: String,
    pub(crate) text: String,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentProjection {
    pub(crate) thread_id: String,
    pub(crate) checkpoint_id: String,
    pub(crate) messages: Vec<AgentProjectedMessage>,
    pub(crate) accepted_input_ids: Vec<String>,
    pub(crate) plan: Vec<AgentPlanItem>,
    pub(crate) model_checkpoints: Vec<super::models::AgentModelCheckpoint>,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentContentDelta {
    pub(crate) identity: super::control::AgentExecutionIdentity,
    pub(crate) message_id: String,
    pub(crate) text: String,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentPlanItem {
    pub(crate) content: String,
    pub(crate) state: AgentPlanState,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "snake_case")]
pub(crate) enum AgentPlanState {
    Pending,
    InProgress,
    Completed,
}
