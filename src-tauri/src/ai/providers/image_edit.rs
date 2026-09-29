use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use super::grok::GrokFileUploadAdapter;
use crate::ai::{
    contracts::FileUploadCapability,
    transport::{http_client, response, SecurityContext},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ImageEditApi {
    Grok,
    Doubao,
    OpenRouter,
    SiliconFlow,
}

#[derive(Deserialize)]
struct ImageResponse {
    data: Vec<ImageData>,
}

#[derive(Deserialize)]
struct SiliconFlowImageResponse {
    images: Vec<ImageData>,
}

#[derive(Deserialize)]
struct ImageData {
    b64_json: Option<String>,
    url: Option<String>,
}

pub(crate) async fn edit_source(
    api: ImageEditApi,
    ctx: &SecurityContext,
    image: &[u8],
    prompt: &str,
    cancellation: &CancellationToken,
) -> Result<Vec<u8>, String> {
    let endpoint = ctx.endpoint.trim_end_matches('/');
    let (path, body) = match api {
        ImageEditApi::Grok => {
            let upload = GrokFileUploadAdapter.upload_ephemeral_file(
                ctx,
                image.to_vec(),
                "image/png",
                "source.png",
            );
            let uploaded = tokio::select! {
                _ = cancellation.cancelled() => return Err("Image operation was cancelled".into()),
                result = upload => result.map_err(|error| error.to_string())?,
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
        ImageEditApi::Doubao => (
            "images/generations",
            serde_json::json!({
                "model": ctx.model,
                "prompt": prompt,
                "image": data_uri(image),
                "response_format": "url",
            }),
        ),
        ImageEditApi::OpenRouter => (
            "images",
            serde_json::json!({
                "model": ctx.model,
                "prompt": prompt,
                "input_references": [{
                    "type": "image_url",
                    "image_url": {"url": data_uri(image)},
                }],
            }),
        ),
        ImageEditApi::SiliconFlow => (
            "images/generations",
            serde_json::json!({
                "model": ctx.model,
                "prompt": prompt,
                "image": data_uri(image),
            }),
        ),
    };
    let send = http_client()
        .post(format!("{endpoint}/{path}"))
        .bearer_auth(ctx.require_api_key().map_err(|error| error.to_string())?)
        .json(&body)
        .send();
    let result = tokio::select! {
        _ = cancellation.cancelled() => return Err("Image operation was cancelled".into()),
        response = send => response.map_err(|error| error.to_string())?,
    };
    if !result.status().is_success() {
        return Err(response::read_text(result)
            .await
            .map_err(|error| error.to_string())?);
    }
    let image = match api {
        ImageEditApi::SiliconFlow => {
            let response: SiliconFlowImageResponse =
                response::read_json(result, response::MAX_IMAGE_JSON_BYTES)
                    .await
                    .map_err(|error| error.to_string())?;
            response.images.into_iter().next()
        }
        ImageEditApi::Grok | ImageEditApi::Doubao | ImageEditApi::OpenRouter => {
            let response: ImageResponse =
                response::read_json(result, response::MAX_IMAGE_JSON_BYTES)
                    .await
                    .map_err(|error| error.to_string())?;
            response.data.into_iter().next()
        }
    }
    .ok_or("Image edit returned no image")?;
    if let Some(encoded) = image.b64_json {
        return response::decode_image(&encoded).map_err(|error| error.to_string());
    }
    let url = image.url.ok_or("Image edit returned no image")?;
    let download = http_client().get(url).send();
    let result = tokio::select! {
        _ = cancellation.cancelled() => return Err("Image operation was cancelled".into()),
        response = download => response.map_err(|error| error.to_string())?,
    };
    let result = result
        .error_for_status()
        .map_err(|error| error.to_string())?;
    response::read_bytes(result, response::MAX_IMAGE_BYTES)
        .await
        .map_err(|error| error.to_string())
}

fn data_uri(image: &[u8]) -> String {
    format!("data:image/png;base64,{}", STANDARD.encode(image))
}
