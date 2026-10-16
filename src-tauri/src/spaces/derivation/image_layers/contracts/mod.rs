use serde::Deserialize;
use serde_json::Value;

pub const MIN_LAYER_COUNT: usize = 2;
/// Discovery decides how many layers a source carries, so the ceiling only bounds how many output
/// nodes one split may create. It is the worker's own discovery ceiling: the host cannot publish a
/// layer the worker cannot rank.
pub const MAX_LAYER_COUNT: usize =
    crate::ai::contracts::image_compute::MAX_DISCOVERED_LAYERS as usize;

/// One split request. `id` is the Run that owns it, so the outputs and their derivation rows are
/// named after it and the split settles against a single authority. `output_node_id` is the node
/// the Run reserved before discovery ran, which the first layer publishes into.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SplitImageLayersRequest {
    pub id: String,
    pub space_id: String,
    pub output_node_id: String,
    pub source_node_id: String,
    pub source_revision: String,
    pub source_edit: Value,
}

pub fn validate(request: &SplitImageLayersRequest) -> Result<(), String> {
    identity(&request.id, "Image layer split")?;
    identity(&request.space_id, "Space")?;
    identity(&request.output_node_id, "Image layer output node")?;
    identity(&request.source_node_id, "Image layer source node")?;
    identity(&request.source_revision, "Image layer source revision")?;
    if !(request.source_edit.is_null() || request.source_edit.is_object()) {
        return Err("Image layer source edit must be an object or null".into());
    }
    Ok(())
}

/// Rejects a discovered count that no longer describes a layer stack.
pub fn validate_count(count: usize) -> Result<(), String> {
    if !(MIN_LAYER_COUNT..=MAX_LAYER_COUNT).contains(&count) {
        return Err("This image did not split into separable layers".into());
    }
    Ok(())
}

/// Identity of one layer node. The first layer publishes into the node the Run reserved before it
/// knew how many layers there would be; the rest are named after the Run, so a replayed
/// settlement cannot publish into a node that belongs to another split.
pub fn layer_node_id(request: &SplitImageLayersRequest, index: usize) -> String {
    match index {
        0 => request.output_node_id.clone(),
        _ => format!("{}-layer-{index}", request.id),
    }
}

pub fn derivation_id(split_id: &str, index: usize) -> String {
    format!("{split_id}:layer-{index}")
}

fn identity(value: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{label} identity is required"));
    }
    if value.len() > 128 {
        return Err(format!("{label} identity exceeds its limit"));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
