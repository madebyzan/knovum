mod archive;
mod planning;

use std::{
    fs,
    path::{Path, PathBuf},
};

use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;
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
    cancellation: &CancellationToken,
) -> Result<(), crate::foundation::app_error::AppError> {
    super::control::check(cancellation)?;
    validation::validate_manifest_version(manifest)?;
    let manifest_assets = super::manifest_assets::parse_cancellable(manifest, cancellation)?;
    let plans = planning::plan_assets(app, manifest, assets, &manifest_assets, cancellation)?;
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
    let mut packaged = archive::write(
        file,
        manifest,
        &index,
        &plans,
        progress,
        total,
        cancellation,
    )
    .and_then(|_| planning::verify_unchanged(&plans, cancellation))
    .and_then(|_| super::control::check(cancellation))
    .and_then(|_| fs::rename(&staging, target).map_err(|error| error.to_string().into()));
    if let Err(primary) = &mut packaged {
        if let Err(error) = fs::remove_file(&staging) {
            if error.kind() != std::io::ErrorKind::NotFound {
                *primary = super::control::with_cleanup_error(
                    primary.clone(),
                    &format!(
                        "failed to remove Space package staging file {}",
                        staging.display()
                    ),
                    error,
                );
            }
        }
    }
    packaged
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

#[cfg(test)]
mod tests;
