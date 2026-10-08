use crate::spaces::knowledge::contracts::{KnowledgeActorKind, RelationKind, RelationStatus};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub struct CompletedDerivationOutput {
    pub node_id: String,
    pub data: Value,
    pub relation: DerivationRelation,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
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
