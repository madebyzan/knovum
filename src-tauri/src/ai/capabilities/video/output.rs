use reqwest::header::{HeaderMap, HeaderValue};
use tokio_util::sync::CancellationToken;

use crate::{
    ai::transport::{response, SecurityContext},
    foundation::app_error::{AppError, AppResult},
    foundation::public_network::{resource_response, same_origin},
};

const MAX_VIDEO_BYTES: usize = 512 * 1024 * 1024;

pub(super) async fn materialize(
    ctx: &SecurityContext,
    source: &str,
    cancellation: &CancellationToken,
) -> AppResult<Vec<u8>> {
    if cancellation.is_cancelled() {
        return Err(AppError::cancelled("Video download cancelled"));
    }
    if let Some(encoded) = source.strip_prefix("data:video/mp4;base64,") {
        return response::decode_base64(encoded, MAX_VIDEO_BYTES);
    }
    let url = reqwest::Url::parse(source)
        .map_err(|error| AppError::new("invalid_response", error.to_string()))?;
    let mut headers = HeaderMap::new();
    if ctx.provider == "gemini"
        && same_origin(
            &url,
            &crate::ai::transport::validate_endpoint(&ctx.endpoint)?,
        )
    {
        headers.insert(
            "x-goog-api-key",
            HeaderValue::from_str(ctx.require_api_key()?)
                .map_err(|_| AppError::invalid("Gemini API key is not a valid request header"))?,
        );
    }
    let download = async {
        let output = resource_response(url, headers)
            .await
            .map_err(|error| AppError::new("provider_request_failed", error))?;
        response::read_bytes(output, MAX_VIDEO_BYTES).await
    };
    tokio::select! {
        biased;
        _ = cancellation.cancelled() => Err(AppError::cancelled("Video download cancelled")),
        bytes = download => bytes,
    }
}

#[cfg(test)]
#[path = "tests/output.rs"]
mod tests;
