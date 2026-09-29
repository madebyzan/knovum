use base64::{engine::general_purpose::STANDARD, Engine};
use tokio_util::sync::CancellationToken;

use super::image_task;
use crate::ai::transport::{http_client, response, SecurityContext};

pub async fn edit_source(
    ctx: &SecurityContext,
    image: &[u8],
    width: u32,
    height: u32,
    instruction: &str,
    cancellation: &CancellationToken,
) -> Result<Vec<u8>, String> {
    if width < 300
        || height < 300
        || 5 * u64::from(width) < 2 * u64::from(height)
        || 2 * u64::from(width) > 5 * u64::from(height)
    {
        return Err("Kling image edit requires both sides to be at least 300px and an aspect ratio between 1:2.5 and 2.5:1".into());
    }
    let body = serde_json::json!({
        "model_name": ctx.model,
        "prompt": format!("In <<<image_1>>>, {instruction}"),
        "image_list": [{"image": STANDARD.encode(image)}],
        "result_type": "single",
    });
    let request = http_client()
        .post(format!(
            "{}/v1/images/omni-image",
            ctx.endpoint.trim_end_matches('/')
        ))
        .bearer_auth(ctx.require_api_key().map_err(|error| error.to_string())?)
        .json(&body)
        .send();
    let result = tokio::select! {
        _ = cancellation.cancelled() => return Err("Image operation was cancelled".into()),
        response = request => response.map_err(|error| error.to_string())?,
    };
    if !result.status().is_success() {
        return Err(response::read_text(result)
            .await
            .map_err(|error| error.to_string())?);
    }
    let submission: serde_json::Value =
        response::read_json(result, response::MAX_CONTROL_JSON_BYTES)
            .await
            .map_err(|error| error.to_string())?;
    if submission["code"].as_i64() != Some(0) {
        return Err(submission["message"]
            .as_str()
            .unwrap_or("Kling image edit submission failed")
            .to_owned());
    }
    let task_id = submission["data"]["task_id"]
        .as_str()
        .ok_or("Kling image edit returned no task ID")?;
    tokio::select! {
        _ = cancellation.cancelled() => Err("Image operation was cancelled".into()),
        result = image_task::poll(ctx, "omni-image", task_id) => result.map_err(|error| error.to_string()),
    }
}
