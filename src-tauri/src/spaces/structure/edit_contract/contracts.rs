use super::SpaceEditCommand;
use crate::spaces::structure::{RelationStyle, StructureLayout, StructureNode};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct SpaceEditSnapshotResponse {
    pub edit_revision: i64,
    pub structure: Vec<StructureNode>,
    pub layouts: Vec<StructureLayout>,
    pub relation_styles: Vec<RelationStyle>,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct ExecuteSpaceEditRequest {
    pub space_id: String,
    pub mutation_id: String,
    pub expected_edit_revision: i64,
    pub command: SpaceEditCommand,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct SpaceEditHistoryRequest {
    pub space_id: String,
    pub expected_edit_revision: i64,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct SpaceEditHistoryReceipt {
    pub applied_at_revision: i64,
    pub selection_suggestion: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum FieldUpdate<T> {
    #[default]
    Keep,
    Set {
        value: T,
    },
    Clear,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct SpacePoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct SpaceRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum AlignmentEdge {
    Left,
    Center,
    Right,
    Top,
    Middle,
    Bottom,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum DistributionAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum ReorderDirection {
    Front,
    Forward,
    Backward,
    Back,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum AutoLayoutDirection {
    Right,
    Down,
}

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct SpaceEditReceipt {
    pub mutation_id: String,
    pub applied_at_revision: i64,
    pub selection_suggestion: Vec<String>,
}
