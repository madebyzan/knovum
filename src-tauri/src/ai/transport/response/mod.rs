use serde::de::DeserializeOwned;

use crate::app_error::{AppError, AppResult};

pub const MAX_IMAGE_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_IMAGE_JSON_BYTES: usize = 96 * 1024 * 1024;
pub const MAX_CONTROL_JSON_BYTES: usize = 1024 * 1024;

pub async fn read_bytes(mut response: reqwest::Response, limit: usize) -> AppResult<Vec<u8>> {
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        return Err(limit_error());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(provider_error)? {
        if bytes.len().saturating_add(chunk.len()) > limit {
            return Err(limit_error());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

pub async fn read_json<T: DeserializeOwned>(
    response: reqwest::Response,
    limit: usize,
) -> AppResult<T> {
    let bytes = read_bytes(response, limit).await?;
    serde_json::from_slice(&bytes)
        .map_err(|error| AppError::new("invalid_response", error.to_string()))
}

fn provider_error(error: reqwest::Error) -> AppError {
    AppError::new("provider_request_failed", error.without_url().to_string())
}
fn limit_error() -> AppError {
    AppError::new(
        "invalid_response",
        "Provider response exceeded its resource limit",
    )
}

#[cfg(test)]
mod tests;

pub async fn read_text(response: reqwest::Response) -> AppResult<String> {
    let bytes = read_bytes(response, MAX_CONTROL_JSON_BYTES).await?;
    String::from_utf8(bytes).map_err(|error| AppError::new("invalid_response", error.to_string()))
}

pub fn decode_image(encoded: &str) -> AppResult<Vec<u8>> {
    decode_base64(encoded, MAX_IMAGE_BYTES)
}

pub(crate) fn decode_base64(encoded: &str, limit: usize) -> AppResult<Vec<u8>> {
    use base64::Engine;
    if encoded.len() > limit.div_ceil(3).saturating_mul(4) {
        return Err(limit_error());
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|error| AppError::new("invalid_response", error.to_string()))?;
    if bytes.len() > limit {
        return Err(limit_error());
    }
    Ok(bytes)
}

pub(crate) async fn download_image(url: &str) -> AppResult<Vec<u8>> {
    let url = reqwest::Url::parse(url)
        .map_err(|error| AppError::new("invalid_response", error.to_string()))?;
    let response = crate::public_network::resource_response(url, reqwest::header::HeaderMap::new())
        .await
        .map_err(|error| AppError::new("provider_request_failed", error))?;
    read_bytes(response, MAX_IMAGE_BYTES).await
}
