use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::super::{
    access::{asset_directory, authorize_source},
    kind::{AssetKind, PreparedAssetState},
    transfer::imported_resource_name,
    PreparedAsset,
};
use super::markdown::{import_markdown, is_markdown};
use super::registry::{self, PendingImport};
use crate::file_inspection::{classify_file, modified_millis, ResourceClassification};

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct PreparedSpaceImport {
    pub job_id: Option<String>,
    pub asset: PreparedAsset,
}

pub(super) fn prepare_asset(
    app: &tauri::AppHandle,
    source: PathBuf,
) -> Result<PreparedSpaceImport, String> {
    authorize_source(app, &source)?;
    prepare_owned_asset(app, source, CancellationToken::new())
}

pub(super) fn prepare_clipboard_asset(
    app: &tauri::AppHandle,
    source: PathBuf,
    clipboard_paths: &[String],
) -> Result<PreparedSpaceImport, String> {
    if !clipboard_contains_source(&source, clipboard_paths) {
        return Err("The resource is no longer present on the clipboard".into());
    }
    prepare_owned_asset(app, source, CancellationToken::new())
}

fn clipboard_contains_source(source: &Path, clipboard_paths: &[String]) -> bool {
    clipboard_paths.iter().any(|path| Path::new(path) == source)
}

#[cfg(test)]
mod tests;

pub(super) fn prepare_owned_asset(
    app: &tauri::AppHandle,
    source: PathBuf,
    cancellation: CancellationToken,
) -> Result<PreparedSpaceImport, String> {
    let name = display_name(&source)?;
    let metadata = fs::symlink_metadata(&source).map_err(|error| error.to_string())?;
    if !metadata.is_file() {
        return Err("Only regular files can be imported into the Space".to_owned());
    }
    if is_markdown(&source) {
        return Ok(PreparedSpaceImport {
            job_id: None,
            asset: import_markdown(&source, name)?,
        });
    }
    let classification = classify_file(&source);
    let target = asset_directory(app)?.join(imported_resource_name(&source)?);
    let job_id = Uuid::new_v4().to_string();
    let total_bytes = Some(metadata.len());
    let asset = prepared_asset(
        &source,
        &target,
        name.clone(),
        classification,
        total_bytes,
        &metadata,
    )?;
    registry::insert(
        job_id.clone(),
        PendingImport {
            source,
            target,
            name,
            classification,
            total_bytes,
            cancellation,
        },
    )?;
    Ok(PreparedSpaceImport {
        job_id: Some(job_id),
        asset,
    })
}

fn prepared_asset(
    source: &Path,
    target: &Path,
    name: String,
    classification: ResourceClassification,
    total_bytes: Option<u64>,
    metadata: &fs::Metadata,
) -> Result<PreparedAsset, String> {
    let kind = AssetKind::from_classification(classification);
    let geometry = if classification == ResourceClassification::Image {
        super::geometry::visual_geometry(source, "image")?
    } else {
        None
    };
    let (preview_text, entries) = if classification == ResourceClassification::Archive {
        let entries = crate::file_inspection::archive_entries(source, &name)?;
        let text = crate::file_inspection::entry_search_text(&entries);
        (Some(text), Some(entries))
    } else {
        (None, None)
    };
    Ok(PreparedAsset {
        name,
        path: Some(target.to_string_lossy().into_owned()),
        kind,
        text: None,
        preview_text,
        preview_language: classification.source_language().map(str::to_owned),
        preview_path: None,
        extracted_text: None,
        size: total_bytes,
        modified_at: Some(modified_millis(metadata)?),
        source_revision: None,
        processing_state: Some(PreparedAssetState::Importing),
        intrinsic_width: geometry.map(|value| value.0),
        intrinsic_height: geometry.map(|value| value.1),
        knowledge_chunks: None,
        document_coverage: None,
        entries,
    })
}

fn display_name(source: &Path) -> Result<String, String> {
    source
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| "The selected resource has no usable name".to_owned())
}
