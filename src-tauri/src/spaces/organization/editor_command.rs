use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::contract::StructureCommand;
use crate::spaces::canvas_state::contract::{CanvasNodeFrame, CanvasNodeSizeAuthority, CanvasRelationColor};
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(in crate::spaces) enum EditorCommand {
    Structure { command: StructureCommand, selection: Vec<String> },
    TransformFrames { updates: Vec<FrameUpdate> },
    ArrangeFrames { updates: Vec<FrameUpdate> },
    SetAppearance { #[serde(rename = "itemId")] item_id: String, changes: BTreeMap<AppearanceField, Option<Value>> },
    SetRelationColor { #[serde(rename = "relationId")] relation_id: String, color: Option<CanvasRelationColor> },
    MoveLayer { ids: Vec<String>, mode: LayerMove },
    DeleteSelection { ids: Vec<String>, #[serde(rename = "relationIds")] relation_ids: Vec<String> },
    MeasureText { measurements: Vec<TextMeasurement> },
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct FrameUpdate {
    pub node_id: String, pub frame: CanvasNodeFrame,
    #[serde(skip_serializing_if = "Option::is_none")] pub size_authority: Option<CanvasNodeSizeAuthority>,
    #[serde(skip_serializing_if = "Option::is_none")] pub typography_scale: Option<f64>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct TextMeasurement {
    pub item_id: String, pub width: f64, pub height: f64, pub typography_scale: f64,
    pub content_revision: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::spaces) enum AppearanceField {
    Fill, CustomFill, TextColor, FontSize, TypographyScale, FontWeight, FontStyle,
    TextDecoration, TextAlign, VerticalAlign, Background, BackgroundStyle,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::spaces) enum LayerMove { Front, Forward, Backward, Back }
impl EditorCommand {
    pub(in crate::spaces) fn deletes_subtrees(&self) -> bool {
        matches!(self, Self::DeleteSelection { .. } | Self::Structure { command: StructureCommand::DeleteSubtrees { .. }, .. })
    }
    pub(in crate::spaces) fn requested_relation_deletions(&self) -> &[String] {
        match self { Self::DeleteSelection { relation_ids, .. } => relation_ids, _ => &[] }
    }
}
