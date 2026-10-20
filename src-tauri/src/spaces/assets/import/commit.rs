use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::ipc::Channel;

use super::super::{
    kind::{AssetKind, PreparedAssetState},
    snapshot_to_imported,
    transfer::remove_path,
    PreparedAsset,
};
use super::{
    copy::{copy_resource, CopiedResource},
    registry::{self, PendingImport},
};
use crate::file_inspection::{
    read_html_preview, read_text_preview, InspectedFile, ResourceClassification,
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetImportProgress {
    pub stage: &'static str,
    pub bytes_copied: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bytes: Option<u64>,
}

pub(crate) fn copy_pending_import(
    job_id: &str,
    progress: &Channel<AssetImportProgress>,
) -> Result<PreparedAsset, String> {
    let mut state = ProgressState::default();
    copy_import(job_id, |delta, total| state.report(delta, total, progress))
}

pub(crate) fn copy_owned_import(job_id: &str) -> Result<PreparedAsset, String> {
    copy_import(job_id, |_, _| Ok(()))
}

pub(crate) fn discard_owned_import(job_id: &str) -> Result<(), String> {
    let job = registry::get(job_id)?;
    let release = registry::remove(job_id);
    let cleanup = remove_path(&job.target);
    match (release, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (release, cleanup) => {
            let mut errors = Vec::new();
            if let Err(error) = release {
                errors.push(format!("Import registry release failed: {error}"));
            }
            if let Err(error) = cleanup {
                errors.push(format!("Import target cleanup failed: {error}"));
            }
            Err(errors.join("; "))
        }
    }
}

fn copy_import(
    job_id: &str,
    mut on_bytes: impl FnMut(u64, Option<u64>) -> Result<(), String>,
) -> Result<PreparedAsset, String> {
    let job = registry::get(job_id)?;
    let copied = copy_resource(&job.source, &job.target, &job.cancellation, |delta| {
        on_bytes(delta, job.total_bytes)
    })
    .and_then(|resource| copied_asset(&job, resource));
    let release = registry::remove(job_id);
    match (copied, release) {
        (Ok(asset), Ok(())) => Ok(asset),
        (copy_outcome, released) => {
            let mut failures = Vec::new();
            if let Err(error) = copy_outcome {
                failures.push(error);
            }
            if let Err(error) = released {
                failures.push(format!("Import registry release failed: {error}"));
            }
            if let Err(error) = remove_path(&job.target) {
                failures.push(format!("Import target cleanup failed: {error}"));
            }
            Err(failures.join("; "))
        }
    }
}

struct ProgressState {
    bytes_copied: u64,
    bytes_reported: u64,
    last_report: Instant,
}

impl Default for ProgressState {
    fn default() -> Self {
        Self {
            bytes_copied: 0,
            bytes_reported: 0,
            last_report: Instant::now(),
        }
    }
}

impl ProgressState {
    fn report(
        &mut self,
        delta: u64,
        total_bytes: Option<u64>,
        progress: &Channel<AssetImportProgress>,
    ) -> Result<(), String> {
        self.bytes_copied += delta;
        let complete = total_bytes.is_some_and(|total| self.bytes_copied >= total);
        let due = self.bytes_copied.saturating_sub(self.bytes_reported) >= 4 * 1024 * 1024
            || self.last_report.elapsed() >= Duration::from_millis(100);
        if !complete && !due {
            return Ok(());
        }
        self.bytes_reported = self.bytes_copied;
        self.last_report = Instant::now();
        progress
            .send(AssetImportProgress {
                stage: "copying",
                bytes_copied: self.bytes_copied,
                total_bytes,
            })
            .map_err(|error| error.to_string())
    }
}

fn copied_asset(job: &PendingImport, resource: CopiedResource) -> Result<PreparedAsset, String> {
    let preview = copied_preview(job)?;
    let snapshot = InspectedFile {
        name: job.name.clone(),
        path: job.target.to_string_lossy().into_owned(),
        kind: job.classification.node_kind(),
        preview_text: preview.text,
        preview_language: preview.language,
        size: resource.size,
        modified_at: resource.modified_at,
        entries: preview.entries,
    };
    let mut asset = snapshot_to_imported(snapshot, resource.revision)?;
    asset.processing_state =
        (asset.kind != AssetKind::Audio).then_some(PreparedAssetState::Processing);
    Ok(asset)
}

struct CopiedPreview {
    text: Option<String>,
    language: Option<String>,
    entries: Option<Vec<crate::file_inspection::ArchiveMember>>,
}

fn copied_preview(job: &PendingImport) -> Result<CopiedPreview, String> {
    if let ResourceClassification::SourceCode(language) = job.classification {
        let text = if language == "html" {
            read_html_preview(&job.target).ok()
        } else {
            read_text_preview(&job.target).ok()
        };
        return Ok(CopiedPreview {
            text,
            language: Some(language.to_owned()),
            entries: None,
        });
    }
    if job.classification.is_text_document() {
        return Ok(CopiedPreview {
            text: read_text_preview(&job.target).ok(),
            language: None,
            entries: None,
        });
    }
    if job.classification == ResourceClassification::Archive {
        let entries = crate::file_inspection::archive_entries(&job.target, &job.name)?;
        let text = crate::file_inspection::entry_search_text(&entries);
        return Ok(CopiedPreview {
            text: Some(text),
            language: None,
            entries: Some(entries),
        });
    }
    Ok(CopiedPreview {
        text: None,
        language: None,
        entries: None,
    })
}
