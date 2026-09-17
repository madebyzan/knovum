use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::ipc::Channel;

use super::super::{transfer::remove_path, PreparedAsset};
use super::{
    copy::{copy_resource, CopiedResource},
    registry::{self, PendingImport},
};
use crate::file_inspection::{read_html_preview, read_text_preview, ResourceClassification};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetImportProgress {
    pub stage: &'static str,
    pub bytes_copied: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bytes: Option<u64>,
}

pub(super) fn copy_pending_import(
    job_id: &str,
    progress: &Channel<AssetImportProgress>,
) -> Result<PreparedAsset, String> {
    let job = registry::get(job_id)?;
    let mut state = ProgressState::default();
    let summary = match copy_resource(&job.source, &job.target, &job.cancellation, |delta| {
        state.report(delta, job.total_bytes, progress)
    }) {
        Ok(summary) => summary,
        Err(error) => {
            if let Err(cleanup_err) = remove_path(&job.target) {
                log::warn!(
                    "failed to clean up import target {}: {cleanup_err}",
                    job.target.display()
                );
            }
            if let Err(cleanup_err) = registry::remove(job_id) {
                log::warn!("failed to remove pending import job {job_id}: {cleanup_err}");
            }
            return Err(error);
        }
    };
    let asset = match copied_asset(&job, summary) {
        Ok(asset) => asset,
        Err(error) => {
            if let Err(cleanup_err) = remove_path(&job.target) {
                log::warn!(
                    "failed to clean up invalid imported asset {}: {cleanup_err}",
                    job.target.display()
                );
            }
            registry::remove(job_id)?;
            return Err(error);
        }
    };
    registry::remove(job_id)?;
    Ok(asset)
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

fn copied_asset(job: &PendingImport, summary: CopiedResource) -> Result<PreparedAsset, String> {
    let (preview_text, preview_language, entries) = copied_preview(job);
    let kind = job.classification.node_kind();
    let geometry = if job.classification == ResourceClassification::Image {
        super::geometry::visual_geometry(&job.target, "image")?
    } else {
        None
    };
    Ok(PreparedAsset {
        name: job.name.clone(),
        path: Some(job.target.to_string_lossy().into_owned()),
        kind,
        text: None,
        preview_text,
        preview_language,
        preview_path: None,
        extracted_text: None,
        size: Some(summary.size),
        modified_at: Some(summary.modified_at),
        source_revision: Some(summary.revision),
        processing_state: Some("processing"),
        intrinsic_width: geometry.map(|value| value.0),
        intrinsic_height: geometry.map(|value| value.1),
        knowledge_chunks: None,
        document_coverage: None,
        entries,
    })
}

fn copied_preview(
    job: &PendingImport,
) -> (
    Option<String>,
    Option<String>,
    Option<Vec<crate::file_inspection::InspectedEntry>>,
) {
    if let ResourceClassification::SourceCode(language) = job.classification {
        let text = if language == "html" {
            read_html_preview(&job.target).ok()
        } else {
            read_text_preview(&job.target).ok()
        };
        return (text, Some(language.to_owned()), None);
    }
    if job.classification.is_text_document() {
        return (read_text_preview(&job.target).ok(), None, None);
    }
    if job.classification == ResourceClassification::Archive {
        let entries = crate::file_inspection::archive_entries(&job.target, &job.name).ok();
        let text = entries
            .as_deref()
            .map(crate::file_inspection::entry_search_text);
        return (text, None, entries);
    }
    (None, None, None)
}
