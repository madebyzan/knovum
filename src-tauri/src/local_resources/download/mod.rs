use std::{path::Path, time::Duration};

use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

mod resume;

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
    crate::foundation::http::client_builder()
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
    let mut request =
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
    let offset = resume::offset(path, required_bytes).await?;
    if required_bytes == Some(offset) {
        progress(offset, required_bytes);
        return Ok(());
    }
    if offset > 0 {
        request.headers_mut().insert(
            reqwest::header::RANGE,
            format!("bytes={offset}-")
                .parse()
                .map_err(|_| DownloadError::Terminal("Invalid download offset".into()))?,
        );
    }
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
    let offset = resume::response_offset(&response, offset, required_bytes)?;
    let total = response
        .content_length()
        .and_then(|size| size.checked_add(offset));
    if total.is_some_and(|size| {
        size > maximum_bytes || required_bytes.is_some_and(|expected| size != expected)
    }) {
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
        offset,
    )
    .await
}

async fn write_response(
    mut response: reqwest::Response,
    path: &Path,
    policy: DownloadPolicy,
    cancellation: &CancellationToken,
    progress: &mut impl FnMut(u64, Option<u64>),
    offset: u64,
) -> Result<(), DownloadError> {
    let DownloadPolicy {
        maximum_bytes,
        required_bytes,
        cancellation_message,
    } = policy;
    let total = response
        .content_length()
        .and_then(|size| size.checked_add(offset));
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create(offset == 0).append(offset > 0);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    #[cfg(windows)]
    options.custom_flags(windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT);
    let mut file = options
        .open(path)
        .await
        .map_err(|error| DownloadError::Terminal(error.to_string()))?;
    let metadata = file
        .metadata()
        .await
        .map_err(|error| DownloadError::Terminal(error.to_string()))?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes()
            & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
            != 0
        {
            return Err(DownloadError::Terminal(
                "Download target is a reparse point".into(),
            ));
        }
    }
    if !metadata.is_file() {
        return Err(DownloadError::Terminal(
            "Download target is not a regular file".into(),
        ));
    }
    if offset == 0 {
        file.set_len(0)
            .await
            .map_err(|error| DownloadError::Terminal(error.to_string()))?;
    }
    if file
        .metadata()
        .await
        .map_err(|error| DownloadError::Terminal(error.to_string()))?
        .len()
        != offset
    {
        return Err(DownloadError::Terminal(
            "Partial download changed during transfer".into(),
        ));
    }
    let mut downloaded = offset;
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
