use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageAssistantConversationRow {
    pub id: String,
    pub title: String,
    pub mode: String,
    pub active_branch_id: String,
    pub archived_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageAssistantMessageRow {
    pub id: String,
    pub conversation_id: String,
    pub branch_id: String,
    pub parent_message_id: Option<String>,
    pub role: String,
    pub content: crate::spaces::assistant::AssistantMessageContent,
    pub created_at: i64,
}
