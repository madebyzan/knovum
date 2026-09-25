use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowValueType {
    Value,
    Record,
    Boolean,
    Document,
    Image,
    Audio,
    Video,
    Transcript,
    Subtitle,
    Presentation,
    ResourceRevision,
    Collection,
}

impl WorkflowValueType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Value => "value",
            Self::Record => "record",
            Self::Boolean => "boolean",
            Self::Document => "document",
            Self::Image => "image",
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Transcript => "transcript",
            Self::Subtitle => "subtitle",
            Self::Presentation => "presentation",
            Self::ResourceRevision => "resource_revision",
            Self::Collection => "collection",
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkflowEdge {
    pub id: String,
    pub source_node_id: String,
    pub source_port: WorkflowPort,
    pub target_node_id: String,
    pub target_port: WorkflowPort,
    pub branch: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowPort {
    Input,
    Output,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JoinPolicy {
    AllSucceeded,
    ActiveBranches,
    CollectResults,
}
