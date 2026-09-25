use std::time::{Duration, Instant};

use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use super::media;
use crate::ai::transport::http_client;

const ENDPOINT: &str = "https://queue.fal.run/fal-ai/qwen-image-layered";

#[derive(Deserialize)]
struct Submission {
    request_id: String,
}

#[derive(Deserialize)]
struct Status {
    status: String,
    error: Option<String>,
}

#[derive(Deserialize)]
struct ResultPayload {
    images: Vec<ImageFile>,
}

#[derive(Deserialize)]
struct ImageFile {
    url: String,
}

pub async fn split<F: FnOnce()>(
    api_key: &str,
    source_data_uri: &str,
    layer_count: usize,
    prompt: Option<&str>,
    cancellation: &CancellationToken,
    on_download: F,
) -> Result<Vec<Vec<u8>>, String> {
    let mut body = serde_json::json!({
        "image_url": source_data_uri,
        "num_layers": layer_count,
        "output_format": "png",
        "enable_safety_checker": true,
        "acceleration": "regular"
    });
    if let Some(prompt) = prompt.filter(|value| !value.trim().is_empty()) {
        body["prompt"] = prompt.trim().into();
    }
    let submission = http_client()
        .post(ENDPOINT)
        .header("Authorization", format!("Key {api_key}"))
        .json(&body)
        .send();
    let response = tokio::select! {
        _ = cancellation.cancelled() => return Err("Image layer split was cancelled".into()),
        response = submission => response,
    }
    .map_err(request_error)?
    .error_for_status()
    .map_err(request_error)?
    .json::<Submission>()
    .await
    .map_err(request_error)?;
    validate_request_id(&response.request_id)?;
    wait_for_completion(api_key, &response.request_id, cancellation).await?;
    let result = fetch_result(api_key, &response.request_id, cancellation).await?;
    if result.images.len() != layer_count {
        return Err(format!(
            "fal returned {} layers; {layer_count} were requested",
            result.images.len()
        ));
    }
    on_download();
    let mut layers = Vec::with_capacity(layer_count);
    for image in result.images {
        layers.push(media::download(&image.url, cancellation).await?);
    }
    Ok(layers)
}

async fn wait_for_completion(
    api_key: &str,
    request_id: &str,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(600);
    loop {
        if Instant::now() >= deadline {
            cancel_remote(api_key, request_id).await;
            return Err("fal image layer split timed out".into());
        }
        tokio::select! {
            _ = cancellation.cancelled() => {
                cancel_remote(api_key, request_id).await;
                return Err("Image layer split was cancelled".into());
            }
            _ = tokio::time::sleep(Duration::from_secs(2)) => {}
        }
        let request = http_client()
            .get(format!("{ENDPOINT}/requests/{request_id}/status"))
            .header("Authorization", format!("Key {api_key}"))
            .send();
        let status = tokio::select! {
            _ = cancellation.cancelled() => {
                cancel_remote(api_key, request_id).await;
                return Err("Image layer split was cancelled".into());
            }
            response = request => response,
        }
        .map_err(request_error)?
        .error_for_status()
        .map_err(request_error)?
        .json::<Status>()
        .await
        .map_err(request_error)?;
        match status.status.as_str() {
            "IN_QUEUE" | "IN_PROGRESS" => continue,
            "COMPLETED" if status.error.is_none() => return Ok(()),
            "COMPLETED" => return Err(status.error.unwrap_or_else(|| "fal request failed".into())),
            other => return Err(format!("fal returned an unknown queue status: {other}")),
        }
    }
}

async fn fetch_result(
    api_key: &str,
    request_id: &str,
    cancellation: &CancellationToken,
) -> Result<ResultPayload, String> {
    let request = http_client()
        .get(format!("{ENDPOINT}/requests/{request_id}"))
        .header("Authorization", format!("Key {api_key}"))
        .send();
    tokio::select! {
        _ = cancellation.cancelled() => return Err("Image layer split was cancelled".into()),
        response = request => response,
    }
    .map_err(request_error)?
    .error_for_status()
    .map_err(request_error)?
    .json()
    .await
    .map_err(request_error)
}

async fn cancel_remote(api_key: &str, request_id: &str) {
    let request = http_client()
        .put(format!("{ENDPOINT}/requests/{request_id}/cancel"))
        .header("Authorization", format!("Key {api_key}"))
        .send();
    let _ = tokio::time::timeout(Duration::from_secs(5), request).await;
}

fn validate_request_id(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err("fal returned an invalid request ID".into());
    }
    Ok(())
}

fn request_error(error: impl std::fmt::Display) -> String {
    format!("fal image layer request failed: {error}")
}
