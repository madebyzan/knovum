use std::path::Path;

use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use super::snapshots::SnapshotKind;
use crate::local_resources::preparation::{
    contracts::ResourcePreparationProgress, RESOURCE_PREPARATION_CANCELLED,
};

static DOWNLOADS: Semaphore = Semaphore::const_new(2);

pub(super) async fn snapshot(
    snapshot: SnapshotKind,
    directory: &Path,
    progress: &mut impl FnMut(ResourcePreparationProgress),
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let _permit = tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Err(RESOURCE_PREPARATION_CANCELLED.into()),
        permit = DOWNLOADS.acquire() => permit.map_err(|error| error.to_string())?,
    };
    super::direct_download::snapshot(snapshot, directory, progress, cancellation).await
}
