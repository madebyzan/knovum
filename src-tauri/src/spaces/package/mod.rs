mod manifest_assets;
mod read;
mod restore;
mod validation;
mod write;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

pub(crate) const PACKAGE_VERSION: u64 = 1;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageAsset {
    pub token: String,
    pub path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageImport {
    pub manifest: String,
    pub assets: Vec<ImportedPackageAsset>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedPackageAsset {
    pub token: String,
    pub path: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
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
    destination_path: String,
    manifest: String,
    assets: Vec<PackageAsset>,
    on_progress: Channel<SpacePackageProgress>,
) -> Result<(), String> {
    crate::blocking::run(move || {
        let target = PathBuf::from(destination_path);
        crate::spaces::assets::authorize_destination(&app, &target)?;
        write::write_package(&app, &target, &manifest, &assets, &on_progress)
    })
    .await
}

#[tauri::command]
pub async fn import_space_package(
    app: tauri::AppHandle,
    source_path: String,
    on_progress: Channel<SpacePackageProgress>,
) -> Result<PackageImport, String> {
    crate::blocking::run(move || {
        let source = PathBuf::from(source_path);
        crate::spaces::assets::authorize_source(&app, &source)?;
        read::read_package(&app, &source, &on_progress)
    })
    .await
}

#[tauri::command]
pub async fn restore_space_package<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    request: restore::contracts::RestorePackageRequest,
) -> Result<(), String> {
    restore::restore(&app, request).await
}
