use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(crate) type Extensions = BTreeMap<String, Value>;

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct JsonCanvasDocumentV1 {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<JsonCanvasNodeV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<JsonCanvasEdgeV1>,
    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(tag = "type", rename_all = "lowercase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum JsonCanvasNodeV1 {
    Text(TextNodeV1),
    File(FileNodeV1),
    Link(LinkNodeV1),
    Group(GroupNodeV1),
}

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct TextNodeV1 {
    #[serde(flatten)]
    pub frame: NodeFrameV1,
    pub text: String,
    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct FileNodeV1 {
    #[serde(flatten)]
    pub frame: NodeFrameV1,
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub subpath: Option<String>,
    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct LinkNodeV1 {
    #[serde(flatten)]
    pub frame: NodeFrameV1,
    pub url: String,
    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct GroupNodeV1 {
    #[serde(flatten)]
    pub frame: NodeFrameV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub background_style: Option<BackgroundStyleV1>,
    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct NodeFrameV1 {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct JsonCanvasEdgeV1 {
    pub id: String,
    pub from_node: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub from_side: Option<NodeSideV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub from_end: Option<EdgeEndV1>,
    pub to_node: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub to_side: Option<NodeSideV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub to_end: Option<EdgeEndV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub label: Option<String>,
    #[serde(flatten)]
    pub extensions: Extensions,
}

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum BackgroundStyleV1 {
    Cover,
    Ratio,
    Repeat,
}

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum NodeSideV1 {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum EdgeEndV1 {
    None,
    Arrow,
}

impl JsonCanvasNodeV1 {
    pub(crate) fn frame(&self) -> &NodeFrameV1 {
        match self {
            Self::Text(node) => &node.frame,
            Self::File(node) => &node.frame,
            Self::Link(node) => &node.frame,
            Self::Group(node) => &node.frame,
        }
    }

    pub(crate) fn subpath(&self) -> Option<&str> {
        match self {
            Self::File(node) => node.subpath.as_deref(),
            _ => None,
        }
    }

    pub(crate) fn color(&self) -> Option<&str> {
        self.frame().color.as_deref()
    }
}
