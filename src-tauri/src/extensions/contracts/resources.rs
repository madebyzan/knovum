use super::McpResourceTemplateSelection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum McpResourceSelection {
    Resource {
        uri: String,
    },
    Template {
        selection: McpResourceTemplateSelection,
    },
}

#[derive(Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpResourceContent {
    pub connection_id: String,
    pub connection_revision: String,
    pub catalog_digest: String,
    pub uri: String,
    #[ts(
        type = "Array<{ uri: string; mimeType?: string; text: string } | { uri: string; mimeType?: string; blob: string }>"
    )]
    pub contents: Vec<rmcp::model::ResourceContents>,
    pub digest: String,
}
