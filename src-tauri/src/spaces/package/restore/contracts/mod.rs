mod assistant;
mod derivation;
mod external_conversation;
mod restore_request;
mod run;

use serde::Deserialize;
use serde_json::Value;

pub use assistant::{PackageAssistantConversationRow, PackageAssistantMessageRow};
pub use derivation::PackageDerivationRow;
pub use external_conversation::{PackageExternalAgentConversationRow, PackageExternalAgentTurnRow};
pub use restore_request::RestorePackageRequest;
pub use run::{PackageRunAttemptRow, PackageRunEventRow, PackageRunRow, PackageRunStepRow};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageSpaceRow {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub last_opened_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageObjectTypeRow {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub description: String,
    pub fields: Value,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageNodeRow {
    pub id: String,
    pub kind: String,
    pub data: Value,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageSceneRow {
    pub id: String,
    pub title: String,
    pub viewport_x: f64,
    pub viewport_y: f64,
    pub viewport_zoom: f64,
    pub order_index: i64,
    pub focus_structure_node_ids: Vec<String>,
    pub relation_kinds: Vec<String>,
    pub relation_statuses: Vec<String>,
    pub query_filter: Value,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageRelationRow {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub kind: String,
    pub rationale: String,
    pub status: String,
    pub status_before_invalidation: Option<String>,
    pub confidence: Option<f64>,
    pub actor_kind: String,
    pub actor_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageProvenanceRow {
    pub id: String,
    pub target_kind: String,
    pub target_id: String,
    pub source_node_id: String,
    pub locator: Value,
    pub source_revision: String,
    pub quote: String,
    pub status: String,
    pub actor_kind: String,
    pub actor_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageEventRow {
    pub id: String,
    pub entity_kind: String,
    pub entity_id: String,
    pub event_kind: String,
    pub actor_kind: String,
    pub actor_id: Option<String>,
    pub payload: Value,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageChunkRow {
    pub id: String,
    pub node_id: String,
    pub origin: String,
    pub kind: String,
    pub source_revision: String,
    pub text: String,
    pub locator: Value,
    pub metadata: Value,
    pub created_at: i64,
    pub updated_at: i64,
}

#[cfg(test)]
mod tests;
