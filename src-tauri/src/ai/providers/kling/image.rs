use base64::{engine::general_purpose::STANDARD, Engine};
use futures::future::BoxFuture;
use tokio_util::sync::CancellationToken;

use super::image_task;
use crate::ai::{
    contracts::{
        GeneratedImage, ImageGenerationCapability, ImageGenerationInput, ImageReference,
        ImageReferenceSupport,
    },
    transport::{http_client, response, SecurityContext},
};
use crate::foundation::app_error::{AppError, AppResult};

pub struct KlingImageAdapter;

/// Kling has no file API, so omni-image takes the pixels in the request body.
const MAX_REFERENCE_BYTES: usize = 8 * 1024 * 1024;
/// Kling's omni-image models are the ones verified to accept an image list.
const OMNI_MODELS: [&str; 2] = ["kling-image-o1", "kling-v3-omni"];

impl ImageGenerationCapability for KlingImageAdapter {
    fn reference_support(&self, _provider: &str, model: &str) -> Option<ImageReferenceSupport> {
        OMNI_MODELS
            .contains(&model)
            .then_some(ImageReferenceSupport {
                max_images: 1,
                max_bytes_per_image: MAX_REFERENCE_BYTES,
            })
    }

    fn generate<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a ImageGenerationInput,
        cancellation: &'a CancellationToken,
    ) -> BoxFuture<'a, AppResult<GeneratedImage>> {
        Box::pin(async move {
            let model = if ctx.model.is_empty() {
                "kling-v1"
            } else {
                ctx.model.as_str()
            };
            let (path, payload) = match req.references.first() {
                Some(reference) => ("images/omni-image", omni_payload(model, req, reference)?),
                None => (
                    "images/generations",
                    serde_json::json!({
                        "model": model,
                        "prompt": req.prompt,
                        "aspect_ratio": aspect_ratio(req.require_aspect_ratio()?),
                        "n": 1,
                    }),
                ),
            };

            let submit = http_client()
                .post(format!("{}/v1/{path}", ctx.endpoint.trim_end_matches('/')))
                .bearer_auth(ctx.require_api_key()?)
                .json(&payload)
                .send();
            let submission_response = tokio::select! {
                _ = cancellation.cancelled() => return Err(AppError::cancelled("Image generation was cancelled")),
                response = submit => response.map_err(|e| AppError::new("provider_request_failed", e.to_string()))?,
            };

            if !submission_response.status().is_success() {
                let error_text = response::read_text(submission_response).await?;
                return Err(AppError::new("provider_request_failed", error_text));
            }

            let submission_response_body: serde_json::Value =
                response::read_json(submission_response, response::MAX_CONTROL_JSON_BYTES).await?;
            if submission_response_body["code"]
                .as_i64()
                .unwrap_or_default()
                != 0
            {
                return Err(AppError::new(
                    "provider_request_failed",
                    submission_response_body["message"]
                        .as_str()
                        .unwrap_or("Kling image submission failed"),
                ));
            }

            let task_id = submission_response_body["data"]["task_id"]
                .as_str()
                .ok_or_else(|| {
                    AppError::new(
                        "invalid_response",
                        "Missing task_id in Kling image generation response",
                    )
                })?
                .to_string();

            let task_path = if req.references.is_empty() {
                "generations"
            } else {
                "omni-image"
            };
            let image_bytes = tokio::select! {
                _ = cancellation.cancelled() => return Err(AppError::cancelled("Image generation was cancelled")),
                generated = image_task::poll(ctx, task_path, &task_id) => generated?,
            };
            Ok(GeneratedImage {
                bytes: image_bytes,
                provider: "kling".to_string(),
                model: model.to_string(),
                revised_prompt: None,
            })
        })
    }
}

/// Omni-image refers to its inputs positionally inside the prompt.
fn omni_payload(
    model: &str,
    req: &ImageGenerationInput,
    reference: &ImageReference,
) -> AppResult<serde_json::Value> {
    if reference.width < 300
        || reference.height < 300
        || 5 * u64::from(reference.width) < 2 * u64::from(reference.height)
        || 2 * u64::from(reference.width) > 5 * u64::from(reference.height)
    {
        return Err(AppError::invalid(
            "Kling image editing requires both sides to be at least 300px and an aspect ratio between 1:2.5 and 2.5:1",
        ));
    }
    Ok(serde_json::json!({
        "model_name": model,
        "prompt": format!("In <<<image_1>>>, {}", req.prompt),
        "image_list": [{ "image": STANDARD.encode(&reference.png) }],
        "result_type": "single",
    }))
}

fn aspect_ratio(requested: &str) -> &str {
    match requested {
        "1:1" | "9:16" | "16:9" | "4:3" | "3:4" | "3:2" | "2:3" => requested,
        _ => "16:9",
    }
}
