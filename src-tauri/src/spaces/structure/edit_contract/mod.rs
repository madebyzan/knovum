use super::{LocalTransform, RelationColor, Size, SizingMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod contracts;
pub(crate) use contracts::{
    AlignmentEdge, AutoLayoutFlow, DistributionAxis, ExecuteSpaceEditRequest, FieldUpdate,
    ReorderDirection, SpaceEditHistoryReceipt, SpaceEditHistoryRequest, SpaceEditReceipt,
    SpaceEditSnapshotResponse, SpacePoint, SpaceRect,
};

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(tag = "kind", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum SpaceEditCommand {
    #[serde(rename = "section.create-empty", rename_all = "camelCase")]
    SectionCreateEmpty {
        section_id: String,
        title: String,
        frame: SpaceRect,
        root_order: u32,
    },
    #[serde(rename = "section.draw", rename_all = "camelCase")]
    SectionDraw {
        section_id: String,
        title: String,
        frame: SpaceRect,
        contained_root_ids: Vec<String>,
    },
    #[serde(rename = "section.wrap-selection", rename_all = "camelCase")]
    SectionWrapSelection {
        section_id: String,
        title: String,
        root_ids: Vec<String>,
    },
    #[serde(rename = "section.resize-frame", rename_all = "camelCase")]
    SectionResizeFrame {
        structure_node_id: String,
        frame: SpaceRect,
    },
    #[serde(rename = "section.fit", rename_all = "camelCase")]
    SectionFit { structure_node_id: String },
    #[serde(rename = "section.set-collapsed", rename_all = "camelCase")]
    SectionSetCollapsed {
        structure_node_id: String,
        collapsed: bool,
    },
    #[serde(rename = "section.dissolve", rename_all = "camelCase")]
    SectionDissolve { structure_node_id: String },
    #[serde(rename = "section.update-layout", rename_all = "camelCase")]
    SectionUpdateLayout {
        structure_node_id: String,
        #[ts(optional = nullable)]
        auto_layout: Option<bool>,
        #[serde(default)]
        #[ts(optional, as = "Option<_>")]
        appearance: FieldUpdate<Value>,
    },
    #[serde(rename = "group.create", rename_all = "camelCase")]
    GroupCreate {
        group_id: String,
        root_ids: Vec<String>,
        title: String,
    },
    #[serde(rename = "group.dissolve", rename_all = "camelCase")]
    GroupDissolve { root_ids: Vec<String> },
    #[serde(rename = "structure.move", rename_all = "camelCase")]
    StructureMove {
        root_ids: Vec<String>,
        delta: SpacePoint,
    },
    #[serde(rename = "structure.move-to-container", rename_all = "camelCase")]
    StructureMoveToContainer {
        root_ids: Vec<String>,
        delta: SpacePoint,
        parent_id: Option<String>,
        order: u32,
        #[ts(optional = nullable)]
        expand_section_id: Option<String>,
    },
    #[serde(rename = "structure.transform", rename_all = "camelCase")]
    StructureTransform {
        root_ids: Vec<String>,
        anchor: SpacePoint,
        translation: SpacePoint,
        rotation_delta: f64,
        scale_factor: f64,
    },
    #[serde(rename = "structure.reparent", rename_all = "camelCase")]
    StructureReparent {
        root_ids: Vec<String>,
        parent_id: Option<String>,
        order: u32,
    },
    #[serde(rename = "structure.align", rename_all = "camelCase")]
    StructureAlign {
        root_ids: Vec<String>,
        edge: AlignmentEdge,
    },
    #[serde(rename = "structure.distribute", rename_all = "camelCase")]
    StructureDistribute {
        root_ids: Vec<String>,
        axis: DistributionAxis,
    },
    #[serde(rename = "structure.reorder", rename_all = "camelCase")]
    StructureReorder {
        root_ids: Vec<String>,
        direction: ReorderDirection,
    },
    #[serde(rename = "structure.auto-layout", rename_all = "camelCase")]
    StructureAutoLayout {
        root_ids: Vec<String>,
        flow: AutoLayoutFlow,
        gap: f64,
    },
    #[serde(rename = "structure.clone", rename_all = "camelCase")]
    StructureClone {
        root_ids: Vec<String>,
        target_parent_id: Option<String>,
        offset: SpacePoint,
    },
    #[serde(rename = "structure.delete-subtrees", rename_all = "camelCase")]
    StructureDeleteSubtrees { root_ids: Vec<String> },
    #[serde(rename = "structure.set-lock", rename_all = "camelCase")]
    StructureSetLock { root_ids: Vec<String>, locked: bool },
    #[serde(rename = "container.rename", rename_all = "camelCase")]
    ContainerRename {
        structure_node_id: String,
        title: String,
    },
    #[serde(rename = "content.update-layout", rename_all = "camelCase")]
    ContentUpdateLayout {
        structure_node_id: String,
        #[ts(optional = nullable)]
        transform: Option<LocalTransform>,
        #[ts(optional = nullable)]
        size: Option<Size>,
        #[ts(optional = nullable)]
        sizing_mode: Option<SizingMode>,
        #[serde(default)]
        #[ts(optional, as = "Option<_>")]
        appearance: FieldUpdate<Value>,
        #[serde(default)]
        #[ts(optional, as = "Option<_>")]
        file_subpath: FieldUpdate<String>,
    },
    #[serde(rename = "content.update-heights", rename_all = "camelCase")]
    ContentUpdateHeights { updates: Vec<ContentHeightUpdate> },
    #[serde(rename = "relation.set-style", rename_all = "camelCase")]
    RelationSetStyle {
        relation_id: String,
        color: Option<RelationColor>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct ContentHeightUpdate {
    pub(crate) structure_node_id: String,
    pub(crate) content_revision: Option<String>,
    pub(crate) height: f64,
    pub(crate) measured_width: f64,
    pub(crate) sizing_mode: SizingMode,
    pub(crate) typography_scale: f64,
}

impl SpaceEditCommand {
    pub(crate) fn records_history(&self) -> bool {
        !matches!(self, Self::ContentUpdateHeights { .. })
    }
}
