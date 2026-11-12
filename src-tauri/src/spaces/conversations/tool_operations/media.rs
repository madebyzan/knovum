use crate::{
    agent_runtime::protocol::messages::AgentContentPart,
    ai::{contracts::ExecutionCapability, models::ModelExecutionTarget},
};
use base64::Engine;
use serde_json::{json, Value};

pub(in crate::spaces::conversations) async fn project(
    value: &mut Value,
    target: &ModelExecutionTarget,
) -> Result<Vec<AgentContentPart>, String> {
    let mut media = Vec::new();
    if let Some(content) = value.get_mut("content").and_then(Value::as_array_mut) {
        for content in content {
            block(content, target, &mut media).await?;
        }
    }
    if let Some(contents) = value.get_mut("contents").and_then(Value::as_array_mut) {
        for resource in contents {
            blob(resource, target, &mut media).await?;
        }
    }
    if let Some(messages) = value.get_mut("messages").and_then(Value::as_array_mut) {
        for message in messages {
            if let Some(content) = message.get_mut("content") {
                block(content, target, &mut media).await?;
            }
        }
    }
    Ok(media)
}

async fn block(
    content: &mut Value,
    target: &ModelExecutionTarget,
    media: &mut Vec<AgentContentPart>,
) -> Result<(), String> {
    match content["type"].as_str() {
        Some("image") => image(content, "data", target, media).await,
        Some("audio") => {
            content.as_object_mut().expect("SDK audio").remove("data");
            content["knovumModelNotice"] = json!("Audio bytes are preserved in the original tool receipt. This model binding has no audio input capability; do not claim to have heard them.");
            Ok(())
        }
        Some("resource") => {
            if let Some(resource) = content.get_mut("resource") {
                blob(resource, target, media).await
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}

async fn blob(
    resource: &mut Value,
    target: &ModelExecutionTarget,
    media: &mut Vec<AgentContentPart>,
) -> Result<(), String> {
    if resource.get("blob").is_none() {
        return Ok(());
    }
    if resource["mimeType"]
        .as_str()
        .is_some_and(|mime| mime.starts_with("image/"))
    {
        image(resource, "blob", target, media).await
    } else {
        resource
            .as_object_mut()
            .expect("SDK resource")
            .remove("blob");
        resource["knovumModelNotice"] = json!("Binary bytes are preserved in the original receipt. The model receives metadata only; resource links are not fetched automatically.");
        Ok(())
    }
}

async fn image(
    content: &mut Value,
    field: &str,
    target: &ModelExecutionTarget,
    media: &mut Vec<AgentContentPart>,
) -> Result<(), String> {
    let Some(data) = content
        .as_object_mut()
        .expect("SDK image")
        .remove(field)
        .and_then(|value| value.as_str().map(str::to_owned))
    else {
        return Ok(());
    };
    let mime = content["mimeType"].as_str().unwrap_or("");
    let inspection = deliver(data, mime, target, media).await;
    match inspection {
        Ok(()) => {
            content["knovumModelNotice"] = json!(
                "Image delivered as visual input. Original bytes remain in the tool receipt."
            );
        }
        Err(error) => {
            content["knovumModelNotice"] = json!(error);
        }
    }
    Ok(())
}

async fn deliver(
    data: String,
    mime: &str,
    target: &ModelExecutionTarget,
    media: &mut Vec<AgentContentPart>,
) -> Result<(), String> {
    if !target
        .capabilities
        .contains(&ExecutionCapability::VisionUnderstanding)
    {
        return Err("The frozen model does not support image input".to_owned());
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|_| "MCP image base64 is invalid".to_owned())?;
    super::super::upload::validate_image(&bytes, mime).map_err(|error| error.to_string())?;
    media.push(super::super::context::image_part(bytes, mime).await?);
    Ok(())
}
