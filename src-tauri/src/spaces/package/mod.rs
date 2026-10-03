use crate::app_error::AppError;
mod import;
mod manifest_assets;
mod remap;
pub(crate) use import::reconcile;
mod read;
mod restore;
mod validation;
mod write;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

pub(crate) const PACKAGE_VERSION: u64 = 6;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageAsset {
    pub token: String,
    pub path: String,
}

pub(super) struct PackageImport {
    pub manifest: String,
    pub assets: Vec<ImportedPackageAsset>,
    pub namespace: PathBuf,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedPackageAsset {
    pub token: String,
    pub path: String,
}

#[derive(Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpacePackageProgress {
    stage: &'static str,
    processed: u64,
    total: u64,
    percent: u8,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AssetIndex {
    token: String,
    name: String,
    directory: bool,
    source_revision: Option<String>,
}

pub(super) fn send_progress(
    channel: &Channel<SpacePackageProgress>,
    stage: &'static str,
    processed: u64,
    total: u64,
) -> Result<(), String> {
    let percent = processed
        .saturating_mul(100)
        .checked_div(total)
        .map_or(100, |ratio| ratio.min(100) as u8);
    channel
        .send(SpacePackageProgress {
            stage,
            processed,
            total,
            percent,
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn export_space_package(
    app: tauri::AppHandle,
    space_id: String,
    destination_path: String,
    on_progress: Channel<SpacePackageProgress>,
) -> Result<(), AppError> {
    let snapshot = crate::spaces::queries::package_snapshot::collect(&app, space_id).await?;
    let manifest = serde_json::to_string(&snapshot.manifest).map_err(|e| e.to_string())?;
    Ok(crate::blocking::run(move || {
        let target = PathBuf::from(destination_path);
        crate::spaces::assets::authorize_destination(&app, &target)?;
        write::write_package(&app, &target, &manifest, &snapshot.assets, &on_progress)
    })
    .await?)
}

#[tauri::command]
pub async fn import_space_package(
    app: tauri::AppHandle,
    source_path: String,
    on_progress: Channel<SpacePackageProgress>,
) -> Result<serde_json::Value, AppError> {
    import::execute(app, PathBuf::from(source_path), on_progress).await
}
