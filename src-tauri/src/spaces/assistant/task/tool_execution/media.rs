use serde_json::Value;

use crate::ai::{contracts::ExecutionCapability, models::ModelExecutionTarget};

pub(super) async fn read_node(
    app: &tauri::AppHandle,
    target: &ModelExecutionTarget,
    action_id: &str,
    outcome: &Value,
) -> Result<Vec<crate::agent_runtime::protocol::messages::AssistantContentPart>, String> {
    if action_id != "read-node" || outcome["result"]["kind"] != "image" {
        return Ok(Vec::new());
    }
    if !target
        .capabilities
        .contains(&ExecutionCapability::VisionUnderstanding)
    {
        return Err(
            "assistant-image-read-unavailable: the frozen model does not support images".into(),
        );
    }
    let node = &outcome["result"];
    let revision = node["assetRevision"]
        .as_str()
        .ok_or("assistant-image-read-unavailable: image has no asset revision")?;
    let asset = crate::spaces::assets::node_asset::resolve(app, node, revision).await?;
    let image = super::super::context::image_input_from_file(&asset.path).await?;
    Ok(vec![image])
}
