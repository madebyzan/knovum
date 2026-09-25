use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageExternalAgentConversationRow {
    pub id: String,
    pub run_id: String,
    pub assistant_conversation_id: String,
    pub branch_id: String,
    pub runtime_kind: String,
    pub runtime_session_id: Option<String>,
    pub session_capabilities: Option<serde_json::Value>,
    pub title: String,
    pub state: String,
    pub error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageExternalAgentTurnRow {
    pub id: String,
    pub conversation_id: String,
    pub assistant_message_id: Option<String>,
    pub state: String,
    pub error: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}
