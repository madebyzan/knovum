use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MIN_LAYER_COUNT: usize = 2;
/// Discovery decides how many layers a source carries, so the ceiling only bounds how many output
/// nodes one split may create. Label maps are `u8`, which leaves ample headroom above this.
pub const MAX_LAYER_COUNT: usize = 24;

/// First phase: discover the layers and stage them. No node is created yet, because the layer
/// count is a property of the image rather than an input.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanImageLayersRequest {
    pub plan_id: String,
    pub space_id: String,
    pub source_node_id: String,
    pub source_revision: String,
    pub source_edit: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageLayerPlan {
    pub plan_id: String,
    pub layer_count: usize,
}

/// Second phase: publish the staged layers into the output nodes the caller created for them.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GenerateImageLayersRequest {
    pub generation_id: String,
    pub plan_id: String,
    pub space_id: String,
    pub source_node_id: String,
    pub source_revision: String,
    pub source_edit: Value,
    pub output_node_ids: Vec<String>,
}

pub fn validate_plan(request: &PlanImageLayersRequest) -> Result<(), String> {
    identity(&request.plan_id, "Image layer plan")?;
    source(
        &request.space_id,
        &request.source_node_id,
        &request.source_revision,
        &request.source_edit,
    )
}

pub fn validate(request: &GenerateImageLayersRequest) -> Result<(), String> {
    identity(&request.generation_id, "Image layer generation")?;
    identity(&request.plan_id, "Image layer plan")?;
    source(
        &request.space_id,
        &request.source_node_id,
        &request.source_revision,
        &request.source_edit,
    )?;
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
    Ok(())
}

fn source(
    space_id: &str,
    source_node_id: &str,
    source_revision: &str,
    source_edit: &Value,
) -> Result<(), String> {
    identity(space_id, "Space")?;
    identity(source_node_id, "Image layer source node")?;
    identity(source_revision, "Image layer source revision")?;
    if !(source_edit.is_null() || source_edit.is_object()) {
        return Err("Image layer source edit must be an object or null".into());
    }
    Ok(())
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
