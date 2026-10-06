use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{snapshots::SnapshotFile, sources::HubSource};

#[derive(Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct DownloadRequest<'a> {
    pub directory: &'a Path,
    pub source: HubSource,
    #[schemars(length(min = 1))]
    pub files: Vec<SnapshotFile>,
}

#[derive(Deserialize)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum DownloadEvent {
    Progress { downloaded: u64, total: u64 },
    Complete {},
    Error { message: String },
}
