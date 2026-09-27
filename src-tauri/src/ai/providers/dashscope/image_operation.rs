use crate::ai::transport::response;
use std::time::{Duration, Instant};

use tokio_util::sync::CancellationToken;

use super::image_operation_protocol as protocol;
use crate::ai::transport::{http_client, SecurityContext};

pub enum SourceImageOperation<'a> {
    Upscale { data_uri: &'a str, factor: u8 },
    View { data_uri: &'a str, prompt: &'a str },
}

pub async fn execute_image_operation(
    ctx: &SecurityContext,
    operation: SourceImageOperation<'_>,
    cancellation: &CancellationToken,
) -> Result<Vec<u8>, String> {
    let secret = ctx.require_api_key().map_err(|error| error.to_string())?;
    let endpoint = endpoint(&ctx.endpoint);
    let (path, body) = protocol::request(operation);
    let response = http_client()
        .post(format!("{endpoint}/{path}"))
        .bearer_auth(secret)
        .header("X-DashScope-Async", "enable")
        .json(&body)
        .send()
        .await
        .map_err(provider_error)?;
    let status = response.status();
    let response: protocol::Response =
        response::read_json(response, response::MAX_CONTROL_JSON_BYTES)
            .await
            .map_err(provider_error)?;
    if !status.is_success() {
        return Err(response_error(response.code, response.message));
    }
    let output = response.output.ok_or("DashScope returned no task output")?;
    if let Some(url) = protocol::image_url(&output) {
        return download(url, cancellation).await;
    }
    let task_id = output.task_id.ok_or("DashScope returned no task ID")?;
    poll(ctx, &endpoint, &task_id, cancellation).await
}

async fn poll(
    ctx: &SecurityContext,
    endpoint: &str,
    task_id: &str,
    cancellation: &CancellationToken,
) -> Result<Vec<u8>, String> {
    let deadline = Instant::now() + Duration::from_secs(300);
    loop {
        if Instant::now() >= deadline {
            return Err("DashScope image operation timed out".into());
        }
        tokio::select! {
            _ = cancellation.cancelled() => return Err("Image operation was cancelled".into()),
            _ = tokio::time::sleep(Duration::from_secs(2)) => {}
        }
        let response = http_client()
            .get(format!("{endpoint}/tasks/{task_id}"))
            .bearer_auth(ctx.require_api_key().map_err(|error| error.to_string())?)
            .send()
            .await
            .map_err(provider_error)?
            .error_for_status()
            .map_err(provider_error)?;
        let response: protocol::Response =
            response::read_json(response, response::MAX_CONTROL_JSON_BYTES)
                .await
                .map_err(provider_error)?;
        let output = response
            .output
            .ok_or_else(|| response_error(response.code, response.message))?;
        match output.task_status.as_deref() {
            Some("SUCCEEDED") => {
                return download(
                    protocol::image_url(&output).ok_or("DashScope returned no image URL")?,
                    cancellation,
                )
                .await
            }
            Some("PENDING" | "RUNNING") => continue,
            _ => return Err(response_error(output.code, output.message)),
        }
    }
}

async fn download(url: &str, cancellation: &CancellationToken) -> Result<Vec<u8>, String> {
    tokio::select! {
        _ = cancellation.cancelled() => Err("Image operation was cancelled".into()),
        downloaded_image = crate::web_capture::capture_image_bounded(url, 64 * 1024 * 1024, "aliyuncs.com") => {
            downloaded_image.map(|(_, _, bytes)| bytes)
        }
    }
}

fn endpoint(value: &str) -> String {
    let value = value.trim().trim_end_matches('/');
    let value = value.strip_suffix("/compatible-mode/v1").unwrap_or(value);
    let value = value.strip_suffix("/api/v1").unwrap_or(value);
    format!("{value}/api/v1")
}

fn response_error(code: Option<String>, message: Option<String>) -> String {
    format!(
        "DashScope image operation failed: {} - {}",
        code.as_deref().unwrap_or("unknown"),
        message.as_deref().unwrap_or("request failed")
    )
}

fn provider_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
