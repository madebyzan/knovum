use std::time::{Duration, Instant};

use crate::ai::transport::{http_client, response, SecurityContext};
use crate::foundation::app_error::{AppError, AppResult};

pub(super) async fn poll(
    ctx: &SecurityContext,
    resource: &str,
    task_id: &str,
) -> AppResult<Vec<u8>> {
    let url = format!(
        "{}/v1/images/{resource}/{task_id}",
        ctx.endpoint.trim_end_matches('/')
    );
    let client = http_client();
    let started_at = Instant::now();
    let mut interval = Duration::from_secs(2);
    loop {
        if started_at.elapsed() >= Duration::from_secs(180) {
            return Err(AppError::new(
                "provider_request_failed",
                format!("Kling image task timed out (task_id: {task_id})"),
            ));
        }
        tokio::time::sleep(interval).await;
        if interval < Duration::from_secs(5) {
            interval += Duration::from_secs(1);
        }
        let image_download = client
            .get(&url)
            .bearer_auth(ctx.require_api_key()?)
            .send()
            .await
            .map_err(provider_error)?;
        if !image_download.status().is_success() {
            return Err(AppError::new(
                "provider_request_failed",
                response::read_text(image_download).await?,
            ));
        }
        let body: serde_json::Value =
            response::read_json(image_download, response::MAX_CONTROL_JSON_BYTES).await?;
        if body["code"].as_i64().is_some_and(|code| code != 0) {
            return Err(AppError::new(
                "provider_request_failed",
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
                        AppError::new("invalid_response", "Kling returned no image URL")
                    })?;
                return response::download_image(image_url).await;
            }
            Some("failed") => {
                return Err(AppError::new(
                    "provider_request_failed",
                    body["data"]["task_status_msg"]
                        .as_str()
                        .unwrap_or("Kling image task failed"),
                ));
            }
            Some("submitted" | "processing") => {}
            _ => {
                return Err(AppError::new(
                    "invalid_response",
                    "Kling returned an unknown image task status",
                ));
            }
        }
    }
}

fn provider_error(error: reqwest::Error) -> AppError {
    AppError::new("provider_request_failed", error.to_string())
}
