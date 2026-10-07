use std::{path::Path, time::Duration};

use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

const MAX_PARALLEL_TRANSFERS: usize = 2;
static TRANSFERS: Semaphore = Semaphore::const_new(MAX_PARALLEL_TRANSFERS);

#[derive(Debug)]
pub(crate) enum DownloadError {
    Network(String),
    Http { host: String, status: u16 },
    Terminal(String),
}

impl DownloadError {
    pub(crate) fn message(self) -> String {
        match self {
            Self::Http { host, status } => format!("{host}: HTTP {status}"),
            Self::Network(message) | Self::Terminal(message) => message,
        }
    }
}

pub(crate) fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .https_only(true)
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| error.to_string())
}

pub(crate) struct DownloadPolicy {
    pub(crate) maximum_bytes: u64,
    pub(crate) required_bytes: Option<u64>,
    pub(crate) cancellation_message: &'static str,
}

pub(crate) async fn to_path(
    request: reqwest::RequestBuilder,
    path: &Path,
    policy: DownloadPolicy,
    cancellation: &CancellationToken,
    progress: &mut impl FnMut(u64, Option<u64>),
) -> Result<(), DownloadError> {
    let (client, request) = request.build_split();
    let request =
        request.map_err(|_| DownloadError::Terminal("Invalid download request".into()))?;
    let url = request.url().clone();
    let DownloadPolicy {
        maximum_bytes,
        required_bytes,
        cancellation_message,
    } = policy;
    let _transfer = tokio::select! {
        biased;
        _ = cancellation.cancelled() => {
            return Err(DownloadError::Terminal(cancellation_message.to_string()));
        }
        permit = TRANSFERS.acquire() => permit.map_err(|_| {
            DownloadError::Terminal("Download transfer limit is unavailable".to_string())
        })?,
    };
    let response = tokio::select! {
        biased;
        _ = cancellation.cancelled() => {
            return Err(DownloadError::Terminal(cancellation_message.to_string()));
        }
        outcome = tokio::time::timeout(Duration::from_secs(15), client.execute(request)) => {
            outcome
                .map_err(|_| DownloadError::Network(format!("{}: response timed out", host(&url))))?
                .map_err(|_| DownloadError::Network(format!("{}: connection failed", host(&url))))?
        }
    }
    .error_for_status()
    .map_err(|error| DownloadError::Http {
        host: host(&url).into(),
        status: error.status().map_or(0, |status| status.as_u16()),
    })?;
    let total = response.content_length();
    if total.is_some_and(|size| size > maximum_bytes) {
        return Err(DownloadError::Terminal(
            "Download response exceeds the size limit".to_string(),
        ));
    }
    write_response(
        response,
        path,
        DownloadPolicy {
            maximum_bytes,
            required_bytes,
            cancellation_message,
        },
        cancellation,
        progress,
    )
    .await
}

async fn write_response(
    mut response: reqwest::Response,
    path: &Path,
    policy: DownloadPolicy,
    cancellation: &CancellationToken,
    progress: &mut impl FnMut(u64, Option<u64>),
) -> Result<(), DownloadError> {
    let DownloadPolicy {
        maximum_bytes,
        required_bytes,
        cancellation_message,
    } = policy;
    let total = response.content_length();
    let mut file = tokio::fs::File::create(path)
        .await
        .map_err(|error| DownloadError::Terminal(error.to_string()))?;
    let mut downloaded = 0_u64;
    progress(downloaded, total);
    loop {
        let chunk = tokio::select! {
            biased;
            _ = cancellation.cancelled() => {
                return Err(DownloadError::Terminal(cancellation_message.to_string()));
            }
            outcome = response.chunk() => outcome.map_err(|_| {
                DownloadError::Network("Download body transfer failed".to_string())
            })?,
        };
        let Some(chunk) = chunk else { break };
        downloaded = downloaded.saturating_add(chunk.len() as u64);
        if downloaded > maximum_bytes {
            return Err(DownloadError::Terminal(
                "Download response exceeds the size limit".to_string(),
            ));
        }
        file.write_all(&chunk)
            .await
            .map_err(|error| DownloadError::Terminal(error.to_string()))?;
        progress(downloaded, total);
    }
    if required_bytes.is_some_and(|expected| expected != downloaded)
        || total.is_some_and(|expected| expected != downloaded)
    {
        return Err(DownloadError::Network(
            "Download body ended before its declared size".to_string(),
        ));
    }
    file.sync_all()
        .await
        .map_err(|error| DownloadError::Terminal(error.to_string()))
}

fn host(url: &reqwest::Url) -> &str {
    url.host_str().unwrap_or("unknown")
}

#[cfg(test)]
mod tests;
