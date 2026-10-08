use super::{catalog, snapshots::SnapshotKind};
use crate::local_resources::{
    download,
    preparation::{
        contracts::{ResourcePreparationProgress, ResourcePreparationStage},
        RESOURCE_PREPARATION_CANCELLED,
    },
};
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub(super) async fn snapshot(
    snapshot: SnapshotKind,
    directory: &Path,
    progress: &mut impl FnMut(ResourcePreparationProgress),
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let client = download::client()?;
    let total = snapshot.total_bytes();
    let mut completed = 0;
    for kind in snapshot.models() {
        let descriptor = catalog::descriptor(*kind);
        download::to_path(
            client.get(catalog::direct_url(*kind)),
            &directory.join(descriptor.file),
            download::DownloadPolicy {
                maximum_bytes: descriptor.download_size,
                required_bytes: Some(descriptor.download_size),
                cancellation_message: RESOURCE_PREPARATION_CANCELLED,
            },
            cancellation,
            &mut |bytes, _| {
                progress(ResourcePreparationProgress {
                    stage: ResourcePreparationStage::Downloading,
                    downloaded: completed + bytes,
                    total: Some(total),
                    percent: Some(((completed + bytes) * 100 / total).min(100) as u8),
                })
            },
        )
        .await
        .map_err(|error| error.message())?;
        completed += descriptor.download_size;
    }
    // The existing atomic snapshot installer verifies every SHA256 before publication.
    Ok(())
}
