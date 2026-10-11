use std::path::Path;

use reqwest::{header::CONTENT_RANGE, Response, StatusCode};

use super::DownloadError;

pub(super) async fn offset(path: &Path, required: Option<u64>) -> Result<u64, DownloadError> {
    let Some(required) = required else {
        return Ok(0);
    };
    let metadata = match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(DownloadError::Terminal(error.to_string())),
    };
    if !metadata.is_file() || metadata.len() > required {
        return Err(DownloadError::Terminal(
            "Invalid partial download file".into(),
        ));
    }
    Ok(metadata.len())
}

pub(super) fn response_offset(
    response: &Response,
    requested: u64,
    required: Option<u64>,
) -> Result<u64, DownloadError> {
    if response.status() == StatusCode::OK {
        return Ok(0);
    }
    if response.status() != StatusCode::PARTIAL_CONTENT || requested == 0 {
        return Err(DownloadError::Terminal(
            "Unexpected download response status".into(),
        ));
    }
    let range = response
        .headers()
        .get(CONTENT_RANGE)
        .and_then(|value| value.to_str().ok());
    let parsed = range.and_then(parse_range);
    if !parsed.is_some_and(|(start, end, total)| {
        start == requested
            && end.checked_add(1) == Some(total)
            && required == Some(total)
            && response
                .content_length()
                .is_none_or(|length| length == total - start)
    }) {
        return Err(DownloadError::Terminal(
            "Download range differs from the requested snapshot".into(),
        ));
    }
    Ok(requested)
}

pub(super) fn parse_range(value: &str) -> Option<(u64, u64, u64)> {
    let (range, total) = value.strip_prefix("bytes ")?.split_once('/')?;
    let (start, end) = range.split_once('-')?;
    let (start, end, total) = (start.parse().ok()?, end.parse().ok()?, total.parse().ok()?);
    (start <= end && end < total).then_some((start, end, total))
}

#[cfg(test)]
#[path = "tests/resume.rs"]
mod tests;
