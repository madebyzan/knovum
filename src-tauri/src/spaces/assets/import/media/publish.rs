use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::contracts::{ImportedMedia, MediaOrigin};
use crate::file_inspection::{content_revision, modified_millis};
use crate::media::acquisition::{
    contracts::{AcquisitionProgress, MediaKind},
    metadata::YtDlpMetadata,
    output, playable_asset,
};

/// Where a downloaded media asset lives. The container follows the kind the link asked for, so the
/// path the canvas reserved when the link was dropped is the path the import publishes into.
pub(super) fn reserved_asset(assets: &Path, job_id: Uuid, kind: MediaKind) -> PathBuf {
    assets.join(format!(
        "{job_id}.{}",
        match kind {
            MediaKind::Video => "mp4",
            MediaKind::Audio => "m4a",
        }
    ))
}

pub(super) fn commit(
    assets: &Path,
    workspace: &Path,
    job_id: Uuid,
    kind: MediaKind,
    ffmpeg: &Path,
    progress: &Channel<AcquisitionProgress>,
    cancellation: &CancellationToken,
) -> Result<ImportedMedia, String> {
    let metadata = output::load_metadata(workspace)?;
    let downloaded = output::downloaded_path(workspace, &metadata.filepath)?;
    let downloaded_extension = output::extension(&downloaded)?;
    let playable = playable_asset::prepare(playable_asset::Preparation {
        source: &downloaded,
        workspace,
        ffmpeg,
        metadata: &metadata,
        kind,
        extension: &downloaded_extension,
        progress,
        cancellation,
    })?;
    let target = reserved_asset(assets, job_id, kind);
    fs::rename(&playable, &target)
        .map_err(|error| format!("Unable to commit downloaded media asset: {error}"))?;
    let preview_path = match commit_preview(workspace, assets, job_id) {
        Ok(value) => value,
        Err(error) => {
            let _ = fs::remove_file(&target);
            return Err(error);
        }
    };
    match build_import(metadata, kind, &target, preview_path.clone()) {
        Ok(value) => Ok(value),
        Err(error) => {
            let _ = fs::remove_file(&target);
            if let Some(path) = preview_path {
                let _ = fs::remove_file(path);
            }
            Err(error)
        }
    }
}

fn build_import(
    metadata: YtDlpMetadata,
    media_kind: MediaKind,
    target: &Path,
    preview_path: Option<String>,
) -> Result<ImportedMedia, String> {
    let extension = output::extension(target)?;
    let creator = output::optional_text(metadata.creator(), "creator", 512)?;
    let duration_seconds = metadata.duration_seconds();
    let dimensions = metadata.display_dimensions();
    let title = output::required_text(metadata.title, "title", 512)?;
    let file_metadata = fs::metadata(target).map_err(|error| error.to_string())?;
    let origin = MediaOrigin {
        kind: "web",
        url: output::required_url(metadata.webpage_url, "webpage URL")?,
        provider: output::required_text(metadata.extractor_key, "provider", 128)?,
        external_id: output::required_text(metadata.id, "media identifier", 512)?,
    };
    Ok(ImportedMedia {
        kind: media_kind,
        file_name: format!("{title}.{extension}"),
        title,
        creator,
        duration_seconds,
        intrinsic_width: dimensions.map(|value| value.0),
        intrinsic_height: dimensions.map(|value| value.1),
        asset_path: target.to_string_lossy().into_owned(),
        size: file_metadata.len(),
        modified_at: modified_millis(&file_metadata)?,
        source_revision: content_revision(target)?,
        preview_path,
        origin,
    })
}

fn commit_preview(workspace: &Path, assets: &Path, job_id: Uuid) -> Result<Option<String>, String> {
    let source = workspace.join("thumbnail.jpg");
    if !source.exists() {
        return Ok(None);
    }
    let metadata = fs::symlink_metadata(&source).map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("yt-dlp returned an invalid media thumbnail".into());
    }
    let target = assets.join(format!("{job_id}-preview.jpg"));
    fs::rename(source, &target)
        .map_err(|error| format!("Unable to commit downloaded media thumbnail: {error}"))?;
    Ok(Some(target.to_string_lossy().into_owned()))
}
