use futures::future::BoxFuture;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

use super::GeminiFileUploadAdapter;
use crate::ai::{
    contracts::{
        FileUploadCapability, GeneratedImage, ImageGenerationCapability, ImageGenerationInput,
        ImageReference, ImageReferenceSupport,
    },
    transport::{http_client, response, SecurityContext},
};
use crate::foundation::app_error::{AppError, AppResult};

pub struct GeminiImageAdapter;

/// Gemini accepts inline reference images with a per-file ceiling; the 3.x image
/// models take more than the 2.5 Flash image model.
const MAX_REFERENCE_BYTES: usize = 7 * 1024 * 1024;

#[derive(Deserialize)]
pub(super) struct GeminiResponse {
    candidates: Vec<Candidate>,
}

impl GeminiResponse {
    fn encoded_image(self) -> Option<String> {
        self.candidates
            .into_iter()
            .flat_map(|candidate| candidate.content.parts)
            .find_map(|part| part.inline_data.map(|image| image.data))
    }
}

#[derive(Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponsePart {
    inline_data: Option<InlineMedia>,
}

#[derive(Deserialize)]
struct InlineMedia {
    data: String,
}

fn is_image_model(model: &str) -> bool {
    model.starts_with("gemini-") && model.contains("-image")
}

impl ImageGenerationCapability for GeminiImageAdapter {
    fn reference_support(&self, _provider: &str, model: &str) -> Option<ImageReferenceSupport> {
        is_image_model(model).then(|| ImageReferenceSupport {
            max_images: if model.starts_with("gemini-3") { 14 } else { 3 },
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
            let secret = ctx.require_api_key()?;
            let endpoint = ctx.endpoint.trim_end_matches('/');
            let client = http_client();

            let encoded = if ctx.model.starts_with("imagen") {
                let url = format!("{endpoint}/models/{}:predict", ctx.model);
                let payload = json!({
                    "instances": [{ "prompt": req.prompt }],
                    "parameters": {
                        "sampleCount": 1,
                        "aspectRatio": req.aspect_ratio,
                    }
                });
                let response = client
                    .post(url)
                    .header("x-goog-api-key", secret)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(provider_err)?;
                let data: Value = check_status(response).await?;
                data["predictions"][0]["bytesBase64Encoded"]
                    .as_str()
                    .ok_or_else(|| AppError::new("empty_response", "Imagen returned no image"))?
                    .to_string()
            } else {
                let url = format!("{endpoint}/models/{}:generateContent", ctx.model);
                let response = client
                    .post(url)
                    .header("x-goog-api-key", secret)
                    .json(&request_body(ctx, req, cancellation).await?)
                    .send()
                    .await
                    .map_err(provider_err)?;
                let gemini_response: GeminiResponse = check_status(response).await?;
                gemini_response
                    .encoded_image()
                    .ok_or_else(|| AppError::new("empty_response", "Gemini returned no image"))?
            };

            let bytes = response::decode_image(&encoded)?;

            Ok(GeneratedImage {
                bytes,
                provider: ctx.provider.clone(),
                model: ctx.model.clone(),
                revised_prompt: None,
            })
        })
    }
}

/// A text-to-image request pins the aspect ratio. With references the pixels
/// decide the frame, so the request carries the prompt and the images only.
async fn request_body(
    ctx: &SecurityContext,
    req: &ImageGenerationInput,
    cancellation: &CancellationToken,
) -> AppResult<Value> {
    let mut parts = vec![json!({ "text": req.prompt })];
    for reference in &req.references {
        parts.push(uploaded_part(ctx, reference, cancellation).await?);
    }
    let mut generation_config = json!({ "responseModalities": ["IMAGE"] });
    if req.references.is_empty() {
        generation_config["imageConfig"] = json!({ "aspectRatio": req.aspect_ratio });
    }
    Ok(json!({
        "contents": [{ "parts": parts }],
        "generationConfig": generation_config,
    }))
}

async fn uploaded_part(
    ctx: &SecurityContext,
    reference: &ImageReference,
    cancellation: &CancellationToken,
) -> AppResult<Value> {
    let upload = GeminiFileUploadAdapter.upload_ephemeral_file(
        ctx,
        reference.png.clone(),
        "image/png",
        "reference.png",
    );
    let uploaded = tokio::select! {
        _ = cancellation.cancelled() => return Err(AppError::cancelled("Image generation was cancelled")),
        uploaded = upload => uploaded?,
    };
    let file_uri = uploaded
        .access_url
        .filter(|uri| !uri.is_empty())
        .ok_or_else(|| {
            AppError::new(
                "invalid_response",
                "Gemini image upload returned no file URI",
            )
        })?;
    Ok(json!({ "fileData": { "mimeType": "image/png", "fileUri": file_uri } }))
}

pub(super) async fn check_status<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> AppResult<T> {
    let status = response.status();
    if !status.is_success() {
        let text = response::read_text(response).await?;
        return Err(AppError::new("provider_request_failed", text));
    }
    response::read_json(response, response::MAX_IMAGE_JSON_BYTES).await
}

fn provider_err(e: impl std::fmt::Display) -> AppError {
    AppError::new("provider_request_failed", e.to_string())
}
