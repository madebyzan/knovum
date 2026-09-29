use tokio_util::sync::CancellationToken;

use super::{
    image::{check_status, GeminiResponse},
    GeminiFileUploadAdapter,
};
use crate::ai::{
    contracts::FileUploadCapability,
    transport::{http_client, response, SecurityContext},
};

pub async fn edit_image(
    ctx: &SecurityContext,
    image: &[u8],
    prompt: &str,
    cancellation: &CancellationToken,
) -> Result<Vec<u8>, String> {
    let upload = GeminiFileUploadAdapter.upload_ephemeral_file(
        ctx,
        image.to_vec(),
        "image/png",
        "source.png",
    );
    let uploaded = tokio::select! {
        _ = cancellation.cancelled() => return Err("Image operation was cancelled".into()),
        result = upload => result.map_err(|error| error.to_string())?,
    };
    let file_uri = uploaded
        .access_url
        .filter(|uri| !uri.is_empty())
        .ok_or("Gemini image upload returned no file URI")?;
    let body = serde_json::json!({
        "contents": [{"parts": [
            {"text": prompt},
            {"fileData": {"mimeType": "image/png", "fileUri": file_uri}}
        ]}],
        "generationConfig": {"responseModalities": ["IMAGE"]}
    });
    let request = http_client()
        .post(format!(
            "{}/models/{}:generateContent",
            ctx.endpoint.trim_end_matches('/'),
            ctx.model
        ))
        .header(
            "x-goog-api-key",
            ctx.require_api_key().map_err(|error| error.to_string())?,
        )
        .json(&body)
        .send();
    let result = tokio::select! {
        _ = cancellation.cancelled() => return Err("Image operation was cancelled".into()),
        response = request => response.map_err(|error| error.to_string())?,
    };
    let edited: GeminiResponse = check_status(result)
        .await
        .map_err(|error| error.to_string())?;
    let encoded = edited
        .image_data()
        .ok_or("Gemini image edit returned no image")?;
    response::decode_image(&encoded).map_err(|error| error.to_string())
}
