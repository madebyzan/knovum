use crate::spaces::knowledge::contracts::{KnowledgeActorKind, RelationKind, RelationStatus};
use serde::Serialize;
use serde_json::Value;

/// What a derivation settled into the node it reserved. Lineage is not part of this: a derived
/// node's `derives-from` Relations are created when the node is reserved, so by the time it
/// settles they already exist on the canvas.
#[derive(Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub struct CompletedDerivationOutput {
    pub node_id: String,
    pub data: Value,
}

/// One `derives-from` Relation an output carries. It is written when the output is reserved, so it
/// never leaves the engine as part of a completion result.
pub struct DerivationRelation {
    pub id: String,
    pub space_id: String,
    pub source_id: String,
    pub target_id: String,
    pub kind: RelationKind,
    pub rationale: &'static str,
    pub status: RelationStatus,
    pub actor_kind: KnowledgeActorKind,
    pub created_at: i64,
    pub updated_at: i64,
}
