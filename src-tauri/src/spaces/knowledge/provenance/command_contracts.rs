use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceInput {
    pub(super) id: String,
    pub(super) space_id: String,
    pub(super) target_kind: String,
    pub(super) target_id: String,
    pub(super) source_node_id: String,
    pub(super) locator: Value,
    pub(super) source_revision: String,
    pub(super) quote: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEventRequest {
    pub(super) space_id: String,
    pub(super) entity_kind: Option<String>,
    pub(super) entity_id: Option<String>,
    pub(super) limit: i64,
}
