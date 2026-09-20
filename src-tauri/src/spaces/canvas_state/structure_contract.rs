use serde::{Deserialize, Serialize};
use super::appearance::{CanvasNodeAppearance, CanvasNodeSizeAuthority};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum StructureKind { Content, Group, Section }
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct StructureItem {
    pub id: String, pub kind: StructureKind, pub parent_id: Option<String>, pub order: u32, pub locked: bool,
    #[serde(skip_serializing_if = "Option::is_none")] pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub title: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SpaceStructure { pub version: u8, pub items: Vec<StructureItem> }
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StructureLayoutDocument { pub version: u8, pub items: Vec<StructureLayout> }
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub(crate) enum StructureLayout {
    Content { #[serde(rename = "itemId")] item_id: String, transform: StructureTransform, size: StructureSize,
        #[serde(rename = "sizeAuthority")] size_authority: CanvasNodeSizeAuthority,
        #[serde(skip_serializing_if = "Option::is_none")] appearance: Option<CanvasNodeAppearance>,
        #[serde(rename = "fileSubpath", skip_serializing_if = "Option::is_none")] file_subpath: Option<String> },
    Group { #[serde(rename = "itemId")] item_id: String, transform: StructureTransform },
    Section { #[serde(rename = "itemId")] item_id: String, position: StructurePoint, size: StructureSize, collapsed: bool,
        #[serde(skip_serializing_if = "Option::is_none")] appearance: Option<CanvasNodeAppearance> },
}
impl StructureLayout {
    pub(crate) fn id(&self) -> &str { match self { Self::Content { item_id, .. } | Self::Group { item_id, .. } | Self::Section { item_id, .. } => item_id } }
    pub(crate) fn kind(&self) -> StructureKind { match self { Self::Content { .. } => StructureKind::Content, Self::Group { .. } => StructureKind::Group, Self::Section { .. } => StructureKind::Section } }
    pub(crate) fn local(&self) -> StructureTransform { match self { Self::Content { transform, .. } | Self::Group { transform, .. } => *transform, Self::Section { position, .. } => StructureTransform { x: position.x, y: position.y, rotation: 0.0, scale: 1.0 } } }
    pub(crate) fn appearance(&self) -> Option<&CanvasNodeAppearance> { match self { Self::Content { appearance, .. } | Self::Section { appearance, .. } => appearance.as_ref(), Self::Group { .. } => None } }
}
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StructureTransform { pub x: f64, pub y: f64, pub rotation: f64, pub scale: f64 }
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StructureSize { pub width: f64, pub height: f64 }
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StructurePoint { pub x: f64, pub y: f64 }
