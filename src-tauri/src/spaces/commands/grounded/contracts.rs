use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GroundedExchangeMessageRequest {
    pub id: String,
    pub content: String,
    pub citations: Value,
    pub provider: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GroundedThreadCreationRequest {
    pub scope: String,
    pub node_ids: Vec<String>,
    pub title: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GroundedExchangeRequest {
    pub thread_id: Option<String>,
    pub new_thread: Option<GroundedThreadCreationRequest>,
    pub space_id: String,
    pub user: GroundedExchangeMessageRequest,
    pub assistant: GroundedExchangeMessageRequest,
}
