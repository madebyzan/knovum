mod conversations;
mod derivation;
mod external_conversation;
mod restore_request;
mod run;
mod speech;

use serde::Deserialize;
use serde_json::Value;

pub use conversations::{PackageConversation, PackageConversationMessage};
pub use derivation::PackageDerivation;
pub use external_conversation::{PackageExternalAgentConversation, PackageExternalAgentTurn};
pub use restore_request::RestorePackageRequest;
pub use run::{PackageRun, PackageRunAttempt, PackageRunEvent, PackageRunStep};
pub use speech::{
    PackageSpeechProject, PackageSpeechProjectRevision, PackageSpeechRender, PackageSpeechSegment,
    PackageSpeechSourceSeparation, PackageVoiceCandidate, PackageVoiceProfile,
    PackageVoiceRevision,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageSpace {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub last_opened_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageObjectType {
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
pub struct PackageNode {
    pub id: String,
    pub kind: String,
    pub data: Value,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageScene {
    pub id: String,
    pub title: String,
    pub viewport_x: f64,
    pub viewport_y: f64,
    pub viewport_zoom: f64,
    pub order_index: i64,
    pub focus_structure_node_ids: Vec<String>,
    pub relation_kinds: Vec<crate::spaces::knowledge::contracts::RelationKind>,
    pub relation_statuses: Vec<crate::spaces::knowledge::contracts::RelationStatus>,
    pub query_filter: Value,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageRelation {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub kind: crate::spaces::knowledge::contracts::RelationKind,
    pub rationale: String,
    pub status: crate::spaces::knowledge::contracts::RelationStatus,
    pub status_before_invalidation: Option<crate::spaces::knowledge::contracts::RelationStatus>,
    pub confidence: Option<f64>,
    pub actor_kind: crate::spaces::knowledge::contracts::RelationActorKind,
    pub actor_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageProvenance {
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
pub struct PackageEvent {
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
pub struct PackageChunk {
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
