use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Serialize, Deserialize, TS)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum ExternalContentSource {
    McpResource {
        connection_id: String,
        connection_revision: String,
        catalog_digest: String,
        uri: String,
        content_digest: String,
        captured_at: i64,
    },
}
