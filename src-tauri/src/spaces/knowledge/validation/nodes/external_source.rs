use crate::spaces::knowledge::contracts::ExternalContentSource;

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
