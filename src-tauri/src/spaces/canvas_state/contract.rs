use serde::{Deserialize, Serialize};
pub(crate) use super::appearance::*;
pub(crate) use super::structure_contract::*;
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SpaceEditorDocument {
    pub version: u8, pub structure: SpaceStructure, pub layout: StructureLayoutDocument,
    pub viewport: CanvasViewport,
    #[serde(skip_serializing_if = "Option::is_none")] pub relation_styles: Option<Vec<CanvasRelationStyleV1>>,
}
impl SpaceEditorDocument {
    pub(crate) fn empty() -> Self { Self { version: 1, structure: SpaceStructure { version: 1, items: vec![] }, layout: StructureLayoutDocument { version: 1, items: vec![] }, viewport: CanvasViewport { x: 0.0, y: 0.0, zoom: 1.0 }, relation_styles: None } }
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CanvasRelationStyleV1 { pub relation_id: String, pub color: CanvasRelationColor }
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasRelationColor { Blue, Green, Amber, Red, Purple }
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanvasViewport { pub x: f64, pub y: f64, pub zoom: f64 }
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanvasNodeFrame { pub x: f64, pub y: f64, pub width: f64, pub height: f64, pub rotation: f64 }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StoredCanvasState { pub document: SpaceEditorDocument, pub revision: i64 }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SaveCanvasStateRequest { pub space_id: String, pub base_revision: i64, pub document: SpaceEditorDocument }
