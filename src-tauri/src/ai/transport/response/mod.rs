use serde::de::DeserializeOwned;

use crate::ai::error::{AiError, AiErrorCode, AiResult};

pub const MAX_IMAGE_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_IMAGE_JSON_BYTES: usize = 96 * 1024 * 1024;
pub const MAX_CONTROL_JSON_BYTES: usize = 1024 * 1024;

pub async fn read_bytes(mut response: reqwest::Response, limit: usize) -> AiResult<Vec<u8>> {
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
) -> AiResult<T> {
    let bytes = read_bytes(response, limit).await?;
    serde_json::from_slice(&bytes)
        .map_err(|error| AiError::new(AiErrorCode::InvalidResponse, error.to_string()))
}

fn provider_error(error: reqwest::Error) -> AiError {
    AiError::new(AiErrorCode::ProviderRequestFailed, error.to_string())
}
fn limit_error() -> AiError {
    AiError::new(
        AiErrorCode::InvalidResponse,
        "Provider response exceeded its resource limit",
    )
}

#[cfg(test)]
mod tests;

pub async fn read_text(response: reqwest::Response) -> AiResult<String> {
    let bytes = read_bytes(response, MAX_CONTROL_JSON_BYTES).await?;
    String::from_utf8(bytes)
        .map_err(|error| AiError::new(AiErrorCode::InvalidResponse, error.to_string()))
}

pub fn decode_image(encoded: &str) -> AiResult<Vec<u8>> {
    use base64::Engine;
    if encoded.len() > MAX_IMAGE_BYTES.div_ceil(3) * 4 {
        return Err(limit_error());
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|error| AiError::new(AiErrorCode::InvalidResponse, error.to_string()))?;
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(limit_error());
    }
    Ok(bytes)
}
