use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const FAL_SECRET_KEY: &str = "integrations.image-layers.fal.api-key";
pub const MIN_LAYER_COUNT: usize = 2;
pub const MAX_LAYER_COUNT: usize = 8;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateImageLayersRequest {
    pub id: String,
    pub space_id: String,
    pub source_node_id: String,
    pub source_revision: String,
    pub source_edit: Value,
    pub output_node_ids: Vec<String>,
    pub prompt: Option<String>,
}

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ImageLayerCapability {
    Ready,
    Blocked { reason: ImageLayerBlockReason },
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageLayerBlockReason {
    MissingCredential,
    SecretDatabaseUnavailable,
}

pub fn validate(request: &CreateImageLayersRequest) -> Result<(), String> {
    if request.id.trim().is_empty()
        || request.space_id.trim().is_empty()
        || request.source_node_id.trim().is_empty()
        || request.source_revision.trim().is_empty()
    {
        return Err("Image layer request identity is required".into());
    }
    if request.id.len() > 96
        || request.space_id.len() > 128
        || request.source_node_id.len() > 128
        || request.source_revision.len() > 128
    {
        return Err("Image layer request identity exceeds its limit".into());
    }
    if !(MIN_LAYER_COUNT..=MAX_LAYER_COUNT).contains(&request.output_node_ids.len()) {
        return Err(format!(
            "Image layer split requires {MIN_LAYER_COUNT} to {MAX_LAYER_COUNT} output nodes"
        ));
    }
    let mut unique = std::collections::HashSet::new();
    if request.output_node_ids.iter().any(|id| {
        id.trim().is_empty()
            || id.len() > 128
            || id == &request.source_node_id
            || !unique.insert(id)
    }) {
        return Err("Image layer output nodes must be unique and differ from the source".into());
    }
    if request
        .prompt
        .as_deref()
        .is_some_and(|prompt| prompt.encode_utf16().count() > 1_000)
    {
        return Err("Image layer prompt exceeds 1000 characters".into());
    }
    if !(request.source_edit.is_null() || request.source_edit.is_object()) {
        return Err("Image layer source edit must be an object or null".into());
    }
    Ok(())
}
