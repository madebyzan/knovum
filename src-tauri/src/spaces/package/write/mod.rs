mod archive;
mod planning;

use std::{
    fs,
    path::{Path, PathBuf},
};

use tauri::ipc::Channel;
use uuid::Uuid;

use super::{
    validation::{self, MAX_MANIFEST, MAX_UNPACKED},
    PackageAsset, SpacePackageProgress,
};

pub fn write_package(
    app: &tauri::AppHandle,
    target: &Path,
    manifest: &str,
    assets: &[PackageAsset],
    progress: &Channel<SpacePackageProgress>,
) -> Result<(), String> {
    validation::validate_manifest_version(manifest)?;
    let manifest_assets = super::manifest_assets::parse(manifest)?;
    let plans = planning::plan_assets(app, manifest, assets, &manifest_assets)?;
    let index = serde_json::to_vec(&plans.iter().map(|plan| &plan.index).collect::<Vec<_>>())
        .map_err(|error| error.to_string())?;
    if index.len() as u64 > MAX_MANIFEST {
        return Err("Space package asset index exceeds the package limit".into());
    }
    let total = manifest.len() as u64
        + index.len() as u64
        + plans.iter().map(|plan| plan.bytes).sum::<u64>();
    if total > MAX_UNPACKED {
        return Err("Space package exceeds export limits".into());
    }
    let staging = staging_path(target, "tmp")?;
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staging)
        .map_err(|error| error.to_string())?;
    let packaged = archive::write(file, manifest, &index, &plans, progress, total)
        .and_then(|_| planning::verify_unchanged(&plans))
        .and_then(|_| finalize_destination(&staging, target));
    if packaged.is_err() {
        if let Err(error) = fs::remove_file(&staging) {
            if error.kind() != std::io::ErrorKind::NotFound {
                log::warn!(
                    "failed to remove Space package staging file {}: {error}",
                    staging.display()
                );
            }
        }
    }
    packaged
}

fn finalize_destination(staging: &Path, target: &Path) -> Result<(), String> {
    fs::rename(staging, target).map_err(|error| error.to_string())
}

fn staging_path(target: &Path, label: &str) -> Result<PathBuf, String> {
    let parent = target
        .parent()
        .ok_or("Space package destination has no parent directory")?;
    let name = target
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or("Space package destination name is not UTF-8")?;
    Ok(parent.join(format!(".{name}.{label}-{}", Uuid::new_v4())))
}
