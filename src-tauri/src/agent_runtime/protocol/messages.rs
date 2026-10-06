use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum AssistantContentPart {
    Text { text: String },
    Image { mime_type: String, data: String },
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantInputMessage {
    pub(crate) id: String,
    pub(crate) role: AssistantInputRole,
    pub(crate) content: Vec<AssistantContentPart>,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AssistantInputRole {
    User,
    Assistant,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantProjectionRequest {
    pub(crate) thread_id: String,
    pub(crate) after_checkpoint_id: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantProjectedMessage {
    pub(crate) message_id: String,
    pub(crate) text: String,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantProjection {
    pub(crate) thread_id: String,
    pub(crate) checkpoint_id: String,
    pub(crate) messages: Vec<AssistantProjectedMessage>,
    pub(crate) accepted_input_ids: Vec<String>,
    pub(crate) plan: Vec<AssistantPlanItem>,
    pub(crate) model_checkpoints: Vec<super::models::AssistantModelCheckpoint>,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantContentDelta {
    pub(crate) identity: super::control::AssistantExecutionIdentity,
    pub(crate) message_id: String,
    pub(crate) text: String,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AssistantPlanItem {
    pub(crate) content: String,
    pub(crate) state: AssistantPlanState,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "snake_case")]
pub(crate) enum AssistantPlanState {
    Pending,
    InProgress,
    Completed,
}
