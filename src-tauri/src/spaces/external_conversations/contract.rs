use crate::ai::contracts::ExternalAgentKind;

pub struct StartExternalConversationRequest {
    pub space_id: String,
    pub runtime_kind: ExternalAgentKind,
    pub title: String,
    pub max_duration_seconds: u32,
    pub input_snapshot: serde_json::Value,
}

pub struct ExternalAgentSessionBinding {
    pub id: String,
    pub run_id: String,
    pub conversation_id: String,
    pub branch_id: String,
}
