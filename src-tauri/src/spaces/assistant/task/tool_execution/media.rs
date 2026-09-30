use serde_json::Value;

use crate::ai::{
    contracts::{ExecutionCapability, ModelContentPart},
    models::{ExecutionTarget, ExecutionTargetSelection},
};

pub(super) async fn read_node(
    app: &tauri::AppHandle,
    target: &ExecutionTarget,
    binding: &ExecutionTargetSelection,
    action_id: &str,
    outcome: &Value,
) -> Result<Vec<ModelContentPart>, String> {
    if action_id != "read-node" || outcome["result"]["kind"] != "image" {
        return Ok(Vec::new());
    }
    crate::ai::settings::resolve(
        app,
        binding.clone(),
        ExecutionCapability::VisionUnderstanding,
    )
    .map_err(|error| {
        format!("assistant-image-read-unavailable: selected model cannot inspect images: {error}")
    })?;
    let node = &outcome["result"];
    let revision = node["assetRevision"]
        .as_str()
        .ok_or("assistant-image-read-unavailable: image has no asset revision")?;
    let asset = crate::spaces::assets::node_asset::resolve(app, node, revision).await?;
    let image = super::super::context::prepare_image_input_from_file(&asset.path, target).await?;
    Ok(vec![image])
}
