use serde::{Deserialize, Serialize};
use crate::spaces::canvas_state::{contract::{StructurePoint, StructureTransform}, geometry::Bounds};
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(in crate::spaces) enum StructureCommand {
    CreateEmptySection { id: String, title: String, bounds: Bounds },
    DrawSection { id: String, title: String, bounds: Bounds, ids: Vec<String> },
    SectionAroundSelection { id: String, title: String, ids: Vec<String> },
    GroupSelection { id: String, title: String, ids: Vec<String> },
    DissolveContainer { id: String },
    MoveItems { ids: Vec<String>, delta: StructurePoint },
    ReparentItems { ids: Vec<String>, #[serde(rename = "parentId")] parent_id: Option<String>, #[serde(rename = "insertIndex")] insert_index: Option<usize> },
    DropItems { ids: Vec<String>, #[serde(rename = "parentId")] parent_id: Option<String>, delta: StructurePoint },
    ResizeSection { id: String, bounds: Bounds },
    FitSection { id: String },
    SetSectionCollapsed { id: String, collapsed: bool },
    RenameContainer { id: String, title: String },
    SetItemsLocked { ids: Vec<String>, locked: bool },
    DeleteSubtrees { ids: Vec<String> },
    TransformGroup { id: String, #[serde(rename = "worldTransform")] world_transform: StructureTransform },
    AlignItems { ids: Vec<String>, axis: Axis, alignment: Alignment },
    DistributeItems { ids: Vec<String>, axis: Axis },
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(in crate::spaces) enum Axis { X, Y }
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(in crate::spaces) enum Alignment { Start, Center, End }
