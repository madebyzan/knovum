use crate::ai::contracts::AgentKind;

pub struct StartExternalConversationRequest {
    pub space_id: String,
    pub runtime_kind: AgentKind,
    pub title: String,
    pub max_duration_seconds: u32,
    pub input_snapshot: serde_json::Value,
}

pub struct AgentSessionBinding {
    pub id: String,
    pub run_id: String,
    pub assistant_conversation_id: String,
    pub branch_id: String,
}
