use super::SpaceEditCommand;
use crate::spaces::structure::{RelationStyle, StructureLayout, StructureNode};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SpaceEditSnapshotResponse {
    pub edit_revision: i64,
    pub structure: Vec<StructureNode>,
    pub layouts: Vec<StructureLayout>,
    pub relation_styles: Vec<RelationStyle>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ExecuteSpaceEditRequest {
    pub space_id: String,
    pub mutation_id: String,
    pub expected_edit_revision: i64,
    pub command: SpaceEditCommand,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SpaceEditHistoryRequest {
    pub space_id: String,
    pub expected_edit_revision: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SpaceEditHistoryReceipt {
    pub applied_at_revision: i64,
    pub selection_suggestion: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub(crate) enum FieldUpdate<T> {
    Keep,
    Set { value: T },
    Clear,
}

impl<T> Default for FieldUpdate<T> {
    fn default() -> Self {
        Self::Keep
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(dead_code)]
#[serde(deny_unknown_fields)]
pub(crate) struct SpacePoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SpaceRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(dead_code)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AlignmentEdge {
    Left,
    Center,
    Right,
    Top,
    Middle,
    Bottom,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(dead_code)]
#[serde(rename_all = "lowercase")]
pub(crate) enum DistributionAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ReorderDirection {
    Front,
    Forward,
    Backward,
    Back,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AutoLayoutDirection {
    Right,
    Down,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SpaceEditReceipt {
    pub mutation_id: String,
    pub applied_at_revision: i64,
    pub selection_suggestion: Vec<String>,
}
