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

pub(super) fn validate(value: Option<&serde_json::Value>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    let source: ExternalContentSource = serde_json::from_value(value.clone())
        .map_err(|error| format!("external-content-source-invalid: {error}"))?;
    match source {
        ExternalContentSource::McpResource {
            connection_id,
            connection_revision,
            catalog_digest,
            uri,
            content_digest,
            captured_at,
        } => {
            if connection_id.is_empty()
                || connection_id.len() > 128
                || connection_revision.is_empty()
                || connection_revision.len() > 128
                || uri.is_empty()
                || uri.len() > 8192
                || captured_at < 0
                || [catalog_digest, content_digest].iter().any(|digest| {
                    digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
                })
            {
                return Err("external-content-source-identity-invalid".into());
            }
        }
    }
    Ok(())
}
