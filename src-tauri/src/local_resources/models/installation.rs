use std::{fs, path::Path};

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::{catalog, download, installation_path, snapshot_directory, snapshots::SnapshotKind};
use crate::local_resources::{
    filesystem,
    preparation::{
        contracts::{ResourcePreparationProgress, ResourcePreparationStage},
        RESOURCE_PREPARATION_CANCELLED,
    },
    verification,
};

pub(crate) fn ready<R: Runtime>(
    app: &AppHandle<R>,
    snapshot: SnapshotKind,
) -> Result<bool, String> {
    for kind in snapshot.models() {
        let descriptor = catalog::descriptor(*kind);
        let path = installation_path(app, descriptor)?;
        let metadata = match fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error.to_string()),
        };
        if !metadata.is_file()
            || !verification::receipt_matches(&path, descriptor.sha256, &metadata)?
        {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) async fn install<R: Runtime>(
    app: &AppHandle<R>,
    snapshot: SnapshotKind,
    progress: &mut impl FnMut(ResourcePreparationProgress),
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let target = snapshot_directory(app, snapshot)?;
    let parent = target
        .parent()
        .ok_or("Model snapshot root is unavailable")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    filesystem::verify_managed_root(
        app,
        parent
            .parent()
            .ok_or("Model resource root is unavailable")?,
    )?;
    filesystem::verify_regular_directory(parent)?;
    let staging = parent.join(format!(".{}.downloading", snapshot.digest()));
    fs::create_dir_all(&staging).map_err(|error| error.to_string())?;
    filesystem::verify_regular_directory(&staging)?;
    download::snapshot(snapshot, &staging, progress, cancellation).await?;
    progress(update(snapshot, ResourcePreparationStage::Verifying));
    let verify_path = staging.clone();
    let verify_cancellation = cancellation.clone();
    crate::foundation::blocking::run(move || {
        record_verification(&verify_path, &snapshot.files(), &verify_cancellation)
    })
    .await?;
    if cancellation.is_cancelled() {
        return Err(RESOURCE_PREPARATION_CANCELLED.into());
    }
    progress(update(snapshot, ResourcePreparationStage::Installing));
    let install_staging = staging.clone();
    let install_target = target.clone();
    let install_parent = parent.to_path_buf();
    let backup_name = format!(".{}.previous", snapshot.digest());
    crate::foundation::blocking::run(move || {
        swap_directory(
            &install_staging,
            &install_target,
            &install_parent,
            &backup_name,
        )
    })
    .await?;
    progress(update(snapshot, ResourcePreparationStage::Ready));
    Ok(())
}

/// Three-step directory swap: move any existing installation aside, move the verified staging
/// directory into place, then remove the superseded copy. If installing the new directory fails,
/// restore the previous installation so a locked file on Windows never leaves both missing.
fn swap_directory(
    staging: &Path,
    target: &Path,
    parent: &Path,
    backup_name: &str,
) -> Result<(), String> {
    let backup = parent.join(backup_name);
    filesystem::remove_owned_directory(&backup, parent)?;
    let moved_previous = if target.exists() {
        fs::rename(target, &backup).map_err(|error| error.to_string())?;
        true
    } else {
        false
    };
    if let Err(error) = fs::rename(staging, target) {
        if moved_previous {
            let _ = fs::rename(&backup, target);
        }
        return Err(error.to_string());
    }
    if moved_previous {
        let _ = filesystem::remove_owned_directory(&backup, parent);
    }
    Ok(())
}

fn record_verification(
    directory: &Path,
    files: &[super::snapshots::SnapshotFile],
    cancellation: &CancellationToken,
) -> Result<(), String> {
    for file in files {
        let path = directory.join(file.path);
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if !metadata.is_file() {
            return Err(format!(
                "Model snapshot contains a non-regular file: {}",
                file.path
            ));
        }
        if metadata.len() != file.size {
            fs::remove_file(&path).map_err(|error| error.to_string())?;
            return Err(format!(
                "Model snapshot file metadata differs: {}",
                file.path
            ));
        }
        if !verification::record_verification(
            &path,
            file.sha256,
            cancellation,
            RESOURCE_PREPARATION_CANCELLED,
        )? {
            fs::remove_file(&path).map_err(|error| error.to_string())?;
            return Err(format!(
                "Model snapshot file failed SHA-256 verification: {}",
                file.path
            ));
        }
    }
    Ok(())
}

pub(super) fn update(
    snapshot: SnapshotKind,
    stage: ResourcePreparationStage,
) -> ResourcePreparationProgress {
    ResourcePreparationProgress {
        stage,
        downloaded: snapshot.download_bytes(),
        total: Some(snapshot.download_bytes()),
        percent: Some(100),
    }
}

#[cfg(test)]
#[path = "tests/installation.rs"]
mod tests;
