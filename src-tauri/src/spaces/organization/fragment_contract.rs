use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::spaces::{canvas_state::contract::{SpaceStructure, StructureLayoutDocument, StructurePoint, CanvasRelationStyleV1}, knowledge::indexing::KnowledgeChunkInput};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct EditorFragment {
    pub structure: SpaceStructure,
    pub layout: StructureLayoutDocument,
    pub nodes: Vec<FragmentNode>,
    pub relations: Vec<FragmentRelation>,
    pub object_types: Vec<FragmentObjectType>,
    pub relation_styles: Vec<CanvasRelationStyleV1>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct FragmentNode {
    pub id: String, pub kind: String, pub data: Value, pub source_chunks: Vec<KnowledgeChunkInput>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct FragmentRelation {
    pub id: String, pub source_id: String, pub target_id: String, pub kind: String,
    pub rationale: String, pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")] pub confidence: Option<f64>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct FragmentObjectType {
    pub id: String, pub name: String, pub icon: String, pub description: String, pub fields: Value,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct FragmentPlacement { pub parent_id: Option<String>, pub delta: StructurePoint }
