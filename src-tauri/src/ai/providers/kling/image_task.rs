use std::time::{Duration, Instant};

use crate::ai::{
    error::{AiError, AiErrorCode, AiResult},
    transport::{http_client, response, SecurityContext},
};

pub(super) async fn poll(
    ctx: &SecurityContext,
    resource: &str,
    task_id: &str,
) -> AiResult<Vec<u8>> {
    let url = format!(
        "{}/v1/images/{resource}/{task_id}",
        ctx.endpoint.trim_end_matches('/')
    );
    let client = http_client();
    let started_at = Instant::now();
    let mut interval = Duration::from_secs(2);
    loop {
        if started_at.elapsed() >= Duration::from_secs(180) {
            return Err(AiError::new(
                AiErrorCode::ProviderRequestFailed,
                format!("Kling image task timed out (task_id: {task_id})"),
            ));
        }
        tokio::time::sleep(interval).await;
        if interval < Duration::from_secs(5) {
            interval += Duration::from_secs(1);
        }
        let result = client
            .get(&url)
            .bearer_auth(ctx.require_api_key()?)
            .send()
            .await
            .map_err(provider_error)?;
        if !result.status().is_success() {
            return Err(AiError::new(
                AiErrorCode::ProviderRequestFailed,
                response::read_text(result).await?,
            ));
        }
        let body: serde_json::Value =
            response::read_json(result, response::MAX_CONTROL_JSON_BYTES).await?;
        if body["code"].as_i64().is_some_and(|code| code != 0) {
            return Err(AiError::new(
                AiErrorCode::ProviderRequestFailed,
                body["message"]
                    .as_str()
                    .unwrap_or("Kling image task failed"),
            ));
        }
        match body["data"]["task_status"].as_str() {
            Some("succeed") => {
                let image_url = body["data"]["task_result"]["images"][0]["url"]
                    .as_str()
                    .ok_or_else(|| {
                        AiError::new(AiErrorCode::InvalidResponse, "Kling returned no image URL")
                    })?;
                let download = client
                    .get(image_url)
                    .send()
                    .await
                    .map_err(provider_error)?
                    .error_for_status()
                    .map_err(provider_error)?;
                return response::read_bytes(download, response::MAX_IMAGE_BYTES).await;
            }
            Some("failed") => {
                return Err(AiError::new(
                    AiErrorCode::ProviderRequestFailed,
                    body["data"]["task_status_msg"]
                        .as_str()
                        .unwrap_or("Kling image task failed"),
                ));
            }
            Some("submitted" | "processing") => {}
            _ => {
                return Err(AiError::new(
                    AiErrorCode::InvalidResponse,
                    "Kling returned an unknown image task status",
                ));
            }
        }
    }
}

fn provider_error(error: reqwest::Error) -> AiError {
    AiError::new(AiErrorCode::ProviderRequestFailed, error.to_string())
}
