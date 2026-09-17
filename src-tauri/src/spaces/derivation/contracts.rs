use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletedDerivationOutput {
    pub node_id: String,
    pub data: Value,
    pub relation: DerivationRelation,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivationRelation {
    pub id: String,
    pub space_id: String,
    pub source_id: String,
    pub target_id: String,
    pub kind: &'static str,
    pub rationale: &'static str,
    pub status: &'static str,
    pub actor_kind: &'static str,
    pub created_at: i64,
    pub updated_at: i64,
}
