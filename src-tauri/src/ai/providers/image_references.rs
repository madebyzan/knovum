use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use super::grok::GrokFileUploadAdapter;
use crate::ai::{
    contracts::{FileUploadCapability, ProviderKind},
    transport::{http_client, response, SecurityContext},
};

/// Where an API offers no file reference the pixels have to travel in the JSON
/// body, which inflates them by about a third, so they are capped well below
/// the providers' request limits.
pub(crate) const MAX_INLINE_REFERENCE_BYTES: usize = 8 * 1024 * 1024;

/// The OpenAI-family request shapes that are verified to accept an image
/// alongside the prompt. Generating from pixels is the same request as
/// generating from text, with the image attached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReferenceApi {
    /// Uploads the file first and references it by ID.
    Grok,
    Doubao,
    OpenRouter,
    SiliconFlow,
}

impl ReferenceApi {
    /// The shape a configured model is verified to speak, if any.
    pub(crate) fn for_model(provider: &str, model: &str) -> Option<Self> {
        match (ProviderKind::parse(provider)?, model) {
            (ProviderKind::Grok, "grok-imagine-image-2.0") => Some(Self::Grok),
            (ProviderKind::Doubao, model) if model.starts_with("doubao-seedream-") => {
                Some(Self::Doubao)
            }
            (ProviderKind::OpenRouter, model) if supports_openrouter_references(model) => {
                Some(Self::OpenRouter)
            }
            (ProviderKind::SiliconFlow, model) if model.starts_with("Qwen/Qwen-Image-Edit") => {
                Some(Self::SiliconFlow)
            }
            _ => None,
        }
    }
}

fn supports_openrouter_references(model: &str) -> bool {
    model.starts_with("openai/gpt-image-")
        || model.starts_with("google/gemini-") && model.contains("-image")
        || model.starts_with("bytedance-seed/seedream-")
        || matches!(
            model,
            "black-forest-labs/flux.2-pro"
                | "black-forest-labs/flux.2-max"
                | "black-forest-labs/flux.2-flex"
                | "black-forest-labs/flux.2-klein-4b"
        )
}

#[derive(Deserialize)]
struct ImageResponse {
    data: Vec<ImageContentReference>,
}

#[derive(Deserialize)]
struct SiliconFlowImageResponse {
    images: Vec<ImageContentReference>,
}

#[derive(Deserialize)]
struct ImageContentReference {
    b64_json: Option<String>,
    url: Option<String>,
}

pub(crate) async fn generate_from_reference(
    api: ReferenceApi,
    ctx: &SecurityContext,
    image: &[u8],
    prompt: &str,
    cancellation: &CancellationToken,
) -> Result<Vec<u8>, String> {
    let endpoint = ctx.endpoint.trim_end_matches('/');
    let (path, body) = match api {
        ReferenceApi::Grok => {
            let upload = GrokFileUploadAdapter.upload_ephemeral_file(
                ctx,
                image.to_vec(),
                "image/png",
                "source.png",
            );
            let uploaded = tokio::select! {
                _ = cancellation.cancelled() => return Err("Image generation was cancelled".into()),
                uploaded = upload => uploaded.map_err(|error| error.to_string())?,
            };
            (
                "images/edits",
                serde_json::json!({
                "model": ctx.model,
                "prompt": prompt,
                "image": {"file_id": uploaded.file_id},
                }),
            )
        }
        ReferenceApi::Doubao => (
            "images/generations",
            serde_json::json!({
                "model": ctx.model,
                "prompt": prompt,
                "image": inline(image),
                "response_format": "url",
            }),
        ),
        ReferenceApi::OpenRouter => (
            "images",
            serde_json::json!({
                "model": ctx.model,
                "prompt": prompt,
                "input_references": [{
                    "type": "image_url",
                    "image_url": {"url": inline(image)},
                }],
            }),
        ),
        ReferenceApi::SiliconFlow => (
            "images/generations",
            serde_json::json!({
                "model": ctx.model,
                "prompt": prompt,
                "image": inline(image),
            }),
        ),
    };
    let send = http_client()
        .post(format!("{endpoint}/{path}"))
        .bearer_auth(ctx.require_api_key().map_err(|error| error.to_string())?)
        .json(&body)
        .send();
    let generated = tokio::select! {
        _ = cancellation.cancelled() => return Err("Image generation was cancelled".into()),
        response = send => response.map_err(|error| error.to_string())?,
    };
    if !generated.status().is_success() {
        return Err(response::read_text(generated)
            .await
            .map_err(|error| error.to_string())?);
    }
    let image = match api {
        ReferenceApi::SiliconFlow => {
            let response: SiliconFlowImageResponse =
                response::read_json(generated, response::MAX_IMAGE_JSON_BYTES)
                    .await
                    .map_err(|error| error.to_string())?;
            response.images.into_iter().next()
        }
        ReferenceApi::Grok | ReferenceApi::Doubao | ReferenceApi::OpenRouter => {
            let response: ImageResponse =
                response::read_json(generated, response::MAX_IMAGE_JSON_BYTES)
                    .await
                    .map_err(|error| error.to_string())?;
            response.data.into_iter().next()
        }
    }
    .ok_or("Image generation returned no image")?;
    if let Some(encoded) = image.b64_json {
        return response::decode_image(&encoded).map_err(|error| error.to_string());
    }
    let url = image.url.ok_or("Image generation returned no image")?;
    tokio::select! {
        biased;
        _ = cancellation.cancelled() => Err("Image generation was cancelled".into()),
        bytes = response::download_image(&url) => bytes.map_err(|error| error.to_string()),
    }
}

/// Only for APIs that accept nothing but an inline image.
fn inline(image: &[u8]) -> String {
    format!("data:image/png;base64,{}", STANDARD.encode(image))
}
