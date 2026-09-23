use tokio_util::sync::CancellationToken;

use crate::ai::transport::http_client;

const MAX_LAYER_BYTES: usize = 20 * 1024 * 1024;

pub async fn download(url: &str, cancellation: &CancellationToken) -> Result<Vec<u8>, String> {
    let url = reqwest::Url::parse(url).map_err(|_| "fal returned an invalid layer URL")?;
    validate_url(&url)?;
    let request = http_client().get(url).send();
    let response = tokio::select! {
        _ = cancellation.cancelled() => return Err("Image layer split was cancelled".into()),
        response = request => response,
    }
    .map_err(request_error)?
    .error_for_status()
    .map_err(request_error)?;
    validate_url(response.url())?;
    if response
        .content_length()
        .is_some_and(|size| size > MAX_LAYER_BYTES as u64)
    {
        return Err("fal layer exceeds the 20 MB limit".into());
    }
    let mut response = response;
    let mut bytes = Vec::with_capacity(response.content_length().unwrap_or(0) as usize);
    loop {
        let chunk = tokio::select! {
            _ = cancellation.cancelled() => return Err("Image layer split was cancelled".into()),
            chunk = response.chunk() => chunk.map_err(request_error)?,
        };
        let Some(chunk) = chunk else { break };
        if bytes.len() + chunk.len() > MAX_LAYER_BYTES {
            return Err("fal layer exceeds the 20 MB limit".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn validate_url(url: &reqwest::Url) -> Result<(), String> {
    let host = url.host_str().unwrap_or_default();
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port_or_known_default() != Some(443)
        || !(host == "fal.media" || host.ends_with(".fal.media"))
    {
        return Err("fal returned an unsafe layer URL".into());
    }
    Ok(())
}

fn request_error(error: impl std::fmt::Display) -> String {
    format!("fal layer download failed: {error}")
}
