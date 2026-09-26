use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct RelationStyle {
    pub relation_id: String,
    pub color: RelationColor,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum RelationColor {
    Blue,
    Green,
    Amber,
    Red,
    Purple,
}

impl RelationColor {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Blue => "blue",
            Self::Green => "green",
            Self::Amber => "amber",
            Self::Red => "red",
            Self::Purple => "purple",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum StructureNode {
    Content {
        id: String,
        parent_id: Option<String>,
        order: u32,
        locked: bool,
        node_id: String,
    },
    Group {
        id: String,
        parent_id: Option<String>,
        order: u32,
        locked: bool,
        title: String,
    },
    Section {
        id: String,
        parent_id: Option<String>,
        order: u32,
        locked: bool,
        title: String,
    },
}

impl StructureNode {
    pub fn id(&self) -> &str {
        match self {
            Self::Content { id, .. } | Self::Group { id, .. } | Self::Section { id, .. } => id,
        }
    }

    pub fn parent_id(&self) -> Option<&str> {
        match self {
            Self::Content { parent_id, .. }
            | Self::Group { parent_id, .. }
            | Self::Section { parent_id, .. } => parent_id.as_deref(),
        }
    }

    pub fn order(&self) -> u32 {
        match self {
            Self::Content { order, .. }
            | Self::Group { order, .. }
            | Self::Section { order, .. } => *order,
        }
    }

    pub fn locked(&self) -> bool {
        match self {
            Self::Content { locked, .. }
            | Self::Group { locked, .. }
            | Self::Section { locked, .. } => *locked,
        }
    }

    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Content { .. } => "content",
            Self::Group { .. } => "group",
            Self::Section { .. } => "section",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct LocalTransform {
    pub x: f64,
    pub y: f64,
    pub rotation: f64,
    pub scale: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum SizeAuthority {
    Content,
    Frame,
}

#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum StructureLayout {
    Content {
        structure_node_id: String,
        transform: LocalTransform,
        size: Size,
        size_authority: SizeAuthority,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        appearance: Option<Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        file_subpath: Option<String>,
    },
    Group {
        structure_node_id: String,
        transform: LocalTransform,
    },
    Section {
        structure_node_id: String,
        position: Point,
        size: Size,
        collapsed: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        appearance: Option<Value>,
    },
}

impl StructureLayout {
    pub fn structure_node_id(&self) -> &str {
        match self {
            Self::Content {
                structure_node_id, ..
            }
            | Self::Group {
                structure_node_id, ..
            }
            | Self::Section {
                structure_node_id, ..
            } => structure_node_id,
        }
    }

    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Content { .. } => "content",
            Self::Group { .. } => "group",
            Self::Section { .. } => "section",
        }
    }
}
