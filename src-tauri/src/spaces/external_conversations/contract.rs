use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ai::contracts::AgentKind;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StartExternalConversationRequest {
    pub space_id: String,
    pub runtime_kind: AgentKind,
    pub title: String,
    pub max_duration_seconds: u32,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "ExternalAgentConversation.ts")]
#[serde(rename_all = "camelCase")]
pub struct ExternalConversationSummary {
    pub id: String,
    pub run_id: String,
    pub space_id: String,
    pub runtime_kind: AgentKind,
    pub runtime_session_id: Option<String>,
    #[ts(type = "unknown")]
    pub session_capabilities: Option<serde_json::Value>,
    pub title: String,
    pub state: String,
    pub error: Option<String>,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "ExternalAgentMessage.ts")]
#[serde(rename_all = "camelCase")]
pub struct ExternalConversationMessage {
    pub id: String,
    pub conversation_id: String,
    pub turn_id: String,
    pub role: String,
    pub content: String,
    pub created_at: i64,
}
