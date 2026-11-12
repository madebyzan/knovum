use tokio_util::sync::CancellationToken;

use crate::{
    ai::{
        contracts::VideoGenerationCapability,
        transport::{response, SecurityContext},
    },
    foundation::app_error::{AppError, AppResult},
    foundation::public_network::resource_response,
};

const MAX_VIDEO_BYTES: usize = 512 * 1024 * 1024;

pub(super) async fn materialize(
    adapter: &dyn VideoGenerationCapability,
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
    let headers = adapter.result_download_headers(ctx, &url)?;
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
