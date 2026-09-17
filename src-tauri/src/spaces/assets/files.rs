use std::{fs, path::PathBuf};

use super::{
    access::{asset_directory, authorize_destination, authorize_spaces_asset},
    snapshot_to_imported,
    transfer::{copy_then, imported_resource_name},
    PreparedAsset,
};
use crate::{
    file_inspection::{content_revision, inspect_file, modified_millis},
    spaces::previews,
};

pub struct RefreshedAsset {
    pub asset: Option<PreparedAsset>,
    pub size: Option<u64>,
    pub modified_at: Option<u64>,
}

pub struct RefreshAssetRequest {
    pub source: PathBuf,
    pub file_name: String,
    pub known_revision: Option<String>,
    pub known_size: Option<u64>,
    pub known_modified_at: Option<u64>,
}

pub fn refresh_asset(
    app: &tauri::AppHandle,
    request: RefreshAssetRequest,
) -> Result<RefreshedAsset, String> {
    let canonical = authorize_spaces_asset(app, &request.source)?;
    let metadata = file_metadata(&canonical)?;
    if metadata == (request.known_size, request.known_modified_at) && canonical.is_file() {
        return Ok(RefreshedAsset::unchanged(metadata));
    }
    let revision = content_revision(&canonical)?;
    if request.known_revision.as_deref() == Some(revision.as_str()) {
        return Ok(RefreshedAsset::unchanged(metadata));
    }
    previews::remove_generated(app, &canonical)?;
    let mut asset = snapshot_to_imported(
        inspect_file(&canonical, &request.file_name)?,
        revision.clone(),
    )?;
    let cancellation = tokio_util::sync::CancellationToken::new();
    previews::enrich_visual(app, &canonical, &mut asset, &cancellation)?;
    super::enrichment::content::extract(app, &canonical, &revision, &mut asset, &cancellation)?;
    Ok(RefreshedAsset {
        size: asset.size,
        modified_at: asset.modified_at,
        asset: Some(asset),
    })
}

impl RefreshedAsset {
    fn unchanged((size, modified_at): (Option<u64>, Option<u64>)) -> Self {
        Self {
            asset: None,
            size,
            modified_at,
        }
    }
}

fn file_metadata(source: &std::path::Path) -> Result<(Option<u64>, Option<u64>), String> {
    if !source.is_file() {
        return Ok((None, None));
    }
    let metadata = fs::metadata(source).map_err(|error| error.to_string())?;
    Ok((Some(metadata.len()), Some(modified_millis(&metadata)?)))
}

pub fn open_asset(app: &tauri::AppHandle, source: PathBuf) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;

    let canonical = authorize_spaces_asset(app, &source)?;
    app.opener()
        .open_path(canonical.to_string_lossy().into_owned(), None::<String>)
        .map_err(|error| error.to_string())
}

pub fn duplicate_asset(app: &tauri::AppHandle, source: PathBuf) -> Result<String, String> {
    let canonical = authorize_spaces_asset(app, &source)?;
    let target = asset_directory(app)?.join(imported_resource_name(&canonical)?);
    copy_then(&canonical, &target, || {
        Ok(target.to_string_lossy().into_owned())
    })
}

pub fn remove_asset(app: &tauri::AppHandle, source: PathBuf) -> Result<(), String> {
    let canonical = authorize_spaces_asset(app, &source)?;
    let parent = canonical.parent().map(PathBuf::from);
    previews::remove_generated(app, &canonical)?;
    if canonical.is_dir() {
        fs::remove_dir_all(&canonical).map_err(|error| error.to_string())?;
    } else {
        fs::remove_file(&canonical).map_err(|error| error.to_string())?;
    }
    cleanup_empty_import_namespace(app, parent.as_deref());
    Ok(())
}

fn cleanup_empty_import_namespace(app: &tauri::AppHandle, parent: Option<&std::path::Path>) {
    let Some(parent) = parent else { return };
    let Ok(root) = asset_directory(app) else {
        return;
    };
    if parent.parent() != Some(root.as_path())
        || !parent
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.starts_with("import-"))
    {
        return;
    }
    match fs::read_dir(parent) {
        Ok(mut entries) => {
            if entries.next().is_none() {
                if let Err(error) = fs::remove_dir(parent) {
                    if error.kind() != std::io::ErrorKind::NotFound {
                        log::debug!(
                            "failed to remove empty Space import namespace {}: {error}",
                            parent.display()
                        );
                    }
                }
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            log::debug!(
                "failed to inspect Space import namespace {}: {error}",
                parent.display()
            );
        }
    }
}

pub fn export_asset(
    app: &tauri::AppHandle,
    source: PathBuf,
    destination: PathBuf,
) -> Result<(), String> {
    let canonical = authorize_spaces_asset(app, &source)?;
    authorize_destination(app, &destination)?;
    if destination.exists() {
        return Err("The export destination already exists".into());
    }
    copy_then(&canonical, &destination, || Ok(()))
}

pub fn export_text(
    app: &tauri::AppHandle,
    destination: PathBuf,
    text: String,
) -> Result<(), String> {
    authorize_destination(app, &destination)?;
    fs::write(destination, text).map_err(|error| error.to_string())
}
