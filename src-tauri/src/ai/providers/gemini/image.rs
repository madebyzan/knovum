use base64::Engine;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};

use crate::ai::{
    contracts::{GeneratedImage, ImageGenerationCapability, ImageGenerationInput},
    error::{AiError, AiErrorCode, AiResult},
    transport::{http_client, SecurityContext},
};

pub struct GeminiImageAdapter;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiImageRequest<'a> {
    contents: [Content<'a>; 1],
    generation_config: GenerationConfig<'a>,
}

#[derive(Serialize)]
struct Content<'a> {
    parts: [TextPart<'a>; 1],
}

#[derive(Serialize)]
struct TextPart<'a> {
    text: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationConfig<'a> {
    response_modalities: [&'static str; 1],
    image_config: ImageConfig<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageConfig<'a> {
    aspect_ratio: &'a str,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
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
    inline_data: Option<InlineData>,
}

#[derive(Deserialize)]
struct InlineData {
    data: String,
}

impl ImageGenerationCapability for GeminiImageAdapter {
    fn generate<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a ImageGenerationInput,
    ) -> BoxFuture<'a, AiResult<GeneratedImage>> {
        Box::pin(async move {
            let secret = ctx.require_api_key()?;
            let endpoint = ctx.endpoint.trim_end_matches('/');
            let client = http_client();

            let encoded = if ctx.model.starts_with("imagen") {
                let url = format!("{endpoint}/models/{}:predict", ctx.model);
                let payload = serde_json::json!({
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
                let data: serde_json::Value = check_status(response).await?;
                data["predictions"][0]["bytesBase64Encoded"]
                    .as_str()
                    .ok_or_else(|| {
                        AiError::new(AiErrorCode::EmptyResponse, "Imagen returned no image")
                    })?
                    .to_string()
            } else {
                let url = format!("{endpoint}/models/{}:generateContent", ctx.model);
                let body = GeminiImageRequest {
                    contents: [Content {
                        parts: [TextPart { text: &req.prompt }],
                    }],
                    generation_config: GenerationConfig {
                        response_modalities: ["IMAGE"],
                        image_config: ImageConfig {
                            aspect_ratio: &req.aspect_ratio,
                        },
                    },
                };
                let response = client
                    .post(url)
                    .header("x-goog-api-key", secret)
                    .json(&body)
                    .send()
                    .await
                    .map_err(provider_err)?;
                let gemini_response: GeminiResponse = check_status(response).await?;
                gemini_response
                    .candidates
                    .into_iter()
                    .flat_map(|item| item.content.parts)
                    .find_map(|part| part.inline_data.map(|d| d.data))
                    .ok_or_else(|| {
                        AiError::new(AiErrorCode::EmptyResponse, "Gemini returned no image")
                    })?
            };

            let bytes = base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .map_err(provider_err)?;

            Ok(GeneratedImage {
                bytes,
                provider: ctx.provider.clone(),
                model: ctx.model.clone(),
                revised_prompt: None,
            })
        })
    }
}

async fn check_status<T: serde::de::DeserializeOwned>(response: reqwest::Response) -> AiResult<T> {
    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(AiError::new(AiErrorCode::ProviderRequestFailed, text));
    }
    response.json::<T>().await.map_err(provider_err)
}

fn provider_err(e: impl std::fmt::Display) -> AiError {
    AiError::new(AiErrorCode::ProviderRequestFailed, e.to_string())
}
