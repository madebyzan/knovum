use std::{fs, io::Write, path::Path, time::Duration};

use tokio_util::sync::CancellationToken;

use super::{sources, ModelDescriptor, ModelDownloadProgress};

pub(super) async fn to_path(
    descriptor: ModelDescriptor,
    path: &Path,
    progress: &mut impl FnMut(ModelDownloadProgress) -> Result<(), String>,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let urls = sources::download_urls(descriptor)?;
    let client = reqwest::Client::builder()
        .https_only(true)
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| error.to_string())?;
    let mut failures = Vec::new();
    for url in urls {
        let response = match open_response(&client, vec![url.clone()], cancellation).await {
            Ok(response) => response,
            Err(error) => {
                if cancellation.is_cancelled() {
                    return Err(error);
                }
                failures.push(error);
                continue;
            }
        };
        match write_response(response, descriptor, path, progress, cancellation).await {
            Ok(()) => return Ok(()),
            Err(TransferError::Terminal(error)) => return Err(error),
            Err(TransferError::Network) => {
                failures.push(format!(
                    "{}: body transfer failed",
                    url.host_str().unwrap_or("unknown")
                ));
            }
        }
    }
    Err(format!("Model download failed: {}", failures.join("; ")))
}

enum TransferError {
    Network,
    Terminal(String),
}

impl From<std::io::Error> for TransferError {
    fn from(error: std::io::Error) -> Self {
        Self::Terminal(error.to_string())
    }
}

async fn write_response(
    mut response: reqwest::Response,
    descriptor: ModelDescriptor,
    path: &Path,
    progress: &mut impl FnMut(ModelDownloadProgress) -> Result<(), String>,
    cancellation: &CancellationToken,
) -> Result<(), TransferError> {
    let total = response.content_length();
    // Every source starts a fresh file; bytes from different endpoints are never combined.
    let mut file = fs::File::create(path)?;
    let mut downloaded = 0_u64;
    progress(download_progress(0, total)).map_err(TransferError::Terminal)?;
    loop {
        let chunk = tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(TransferError::Terminal("Model download cancelled".to_string())),
            outcome = response.chunk() => outcome.map_err(|_| TransferError::Network)?,
        };
        let Some(chunk) = chunk else { break };
        if downloaded.saturating_add(chunk.len() as u64) > descriptor.download_size {
            return Err(TransferError::Terminal(
                "Model response exceeds the catalog size".to_string(),
            ));
        }
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        progress(download_progress(downloaded, total)).map_err(TransferError::Terminal)?;
    }
    if downloaded != descriptor.download_size {
        return Err(TransferError::Network);
    }
    file.sync_all()?;
    Ok(())
}

pub(super) async fn open_response(
    client: &reqwest::Client,
    urls: Vec<reqwest::Url>,
    cancellation: &CancellationToken,
) -> Result<reqwest::Response, String> {
    let mut failures = Vec::new();
    for url in urls {
        let outcome = tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err("Model download cancelled".to_string()),
            outcome = tokio::time::timeout(Duration::from_secs(15), client.get(url.clone()).send()) => outcome,
        };
        let reason = match outcome {
            Ok(Ok(response)) => match response.error_for_status() {
                Ok(response) => return Ok(response),
                Err(error) => format!(
                    "HTTP {}",
                    error.status().map_or(0, |status| status.as_u16())
                ),
            },
            Ok(Err(_)) => "connection failed".to_string(),
            Err(_) => "response timed out".to_string(),
        };
        failures.push(format!("{}: {reason}", url.host_str().unwrap_or("unknown")));
    }
    Err(format!(
        "Model sources unavailable: {}",
        failures.join("; ")
    ))
}

pub(super) fn download_progress(downloaded: u64, total: Option<u64>) -> ModelDownloadProgress {
    let percent = total
        .filter(|value| *value > 0)
        .map(|value| ((downloaded.saturating_mul(100) / value).min(100)) as u8);
    ModelDownloadProgress {
        downloaded,
        total,
        percent,
    }
}
