use std::path::Path;

use tokio_util::sync::CancellationToken;

use super::{sources, ModelDescriptor};
use crate::local_resources::{
    download::{self as shared_download, DownloadError},
    preparation::contracts::{ResourcePreparationProgress, ResourcePreparationStage},
};

pub(super) async fn to_path(
    descriptor: ModelDescriptor,
    path: &Path,
    progress: &mut impl FnMut(ResourcePreparationProgress),
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let urls = sources::download_urls(descriptor)?;
    let client = shared_download::client()?;
    let mut failures = Vec::new();
    for url in urls {
        match shared_download::to_path(
            &client,
            url.clone(),
            path,
            shared_download::DownloadPolicy {
                maximum_bytes: descriptor.download_size,
                required_bytes: Some(descriptor.download_size),
                cancellation_message:
                    crate::local_resources::preparation::RESOURCE_PREPARATION_CANCELLED,
            },
            cancellation,
            &mut |downloaded, total| progress(download_progress(downloaded, total)),
        )
        .await
        {
            Ok(()) => return Ok(()),
            Err(DownloadError::Terminal(error)) => return Err(error),
            Err(DownloadError::Network(error)) => failures.push(error),
        }
    }
    Err(format!("Model download failed: {}", failures.join("; ")))
}

pub(super) fn download_progress(
    downloaded: u64,
    total: Option<u64>,
) -> ResourcePreparationProgress {
    let percent = total
        .filter(|value| *value > 0)
        .map(|value| ((downloaded.saturating_mul(100) / value).min(100)) as u8);
    ResourcePreparationProgress {
        stage: ResourcePreparationStage::Downloading,
        downloaded,
        total,
        percent,
    }
}
