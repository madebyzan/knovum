use std::{
    collections::HashSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, Runtime};
use tokio_util::sync::CancellationToken;

use crate::local_resources::{
    download::{self, DownloadPolicy},
    preparation::contracts::{ResourcePreparationProgress, ResourcePreparationStage},
};

mod archive;
mod manifest;

#[derive(Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct KokoroPackageStatus {
    pub installed: bool,
    pub verified: bool,
    pub path: String,
    pub download_size: u64,
    pub license_review_required: bool,
}

pub struct VerifiedKokoroDirectory(PathBuf);

impl VerifiedKokoroDirectory {
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

pub fn installation_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let manifest = manifest::load()?;
    app.path()
        .app_data_dir()
        .map_err(|error| error.to_string())
        .map(|root| {
            root.join("local-resources")
                .join("models")
                .join(manifest.model_id)
        })
}

pub fn status<R: Runtime>(app: &AppHandle<R>) -> Result<KokoroPackageStatus, String> {
    let manifest = manifest::load()?;
    let path = installation_path(app)?;
    let installed = path.is_dir();
    let verified =
        installed && verify_directory(&path, &manifest, &CancellationToken::new()).is_ok();
    Ok(KokoroPackageStatus {
        installed,
        verified,
        path: path.to_string_lossy().into_owned(),
        download_size: manifest.archive_size,
        license_review_required: manifest.license_review.status == "blocked",
    })
}

pub fn require_verified_directory<R: Runtime>(
    app: &AppHandle<R>,
    cancellation: &CancellationToken,
) -> Result<VerifiedKokoroDirectory, String> {
    let manifest = manifest::load()?;
    let path = installation_path(app)?;
    verify_directory(&path, &manifest, cancellation)?;
    Ok(VerifiedKokoroDirectory(path))
}

pub(crate) async fn install<R: Runtime>(
    app: &AppHandle<R>,
    progress: &mut impl FnMut(ResourcePreparationProgress),
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let manifest = manifest::load()?;
    let target = installation_path(app)?;
    if target.is_dir() {
        let verify_target = target.clone();
        let verify_cancel = cancellation.clone();
        progress_update(
            progress,
            ResourcePreparationStage::Verifying,
            0,
            manifest.archive_size,
        );
        if crate::blocking::run(move || {
            verify_directory(&verify_target, &manifest::load()?, &verify_cancel)
        })
        .await
        .is_ok()
        {
            progress_update(
                progress,
                ResourcePreparationStage::Ready,
                manifest.archive_size,
                manifest.archive_size,
            );
            return Ok(());
        }
    }
    let parent = target
        .parent()
        .ok_or("Kokoro package directory is unavailable")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let archive_path = parent.join(format!(".{}.download", manifest.model_id));
    let staging = parent.join(format!(".{}.installing", manifest.model_id));
    remove_file_if_exists(&archive_path)?;
    remove_directory_if_exists(&staging)?;
    let url = reqwest::Url::parse(&manifest.archive_url).map_err(|error| error.to_string())?;
    let client = download::client()?;
    let download = download::to_path(
        client.get(url),
        &archive_path,
        DownloadPolicy {
            maximum_bytes: manifest.archive_size,
            required_bytes: Some(manifest.archive_size),
            cancellation_message: super::super::preparation::RESOURCE_PREPARATION_CANCELLED,
        },
        cancellation,
        &mut |downloaded, _| {
            progress_update(
                progress,
                ResourcePreparationStage::Downloading,
                downloaded,
                manifest.archive_size,
            )
        },
    )
    .await;
    if let Err(error) = download {
        remove_file_if_exists(&archive_path)?;
        return Err(error.message());
    }
    progress_update(
        progress,
        ResourcePreparationStage::Verifying,
        manifest.archive_size,
        manifest.archive_size,
    );
    let archive_for_install = archive_path.clone();
    let staging_for_install = staging.clone();
    let install_cancel = cancellation.clone();
    let archive_sha256 = manifest.archive_sha256.clone();
    crate::blocking::run(move || {
        verify_file(&archive_for_install, &archive_sha256, &install_cancel)?;
        fs::create_dir(&staging_for_install).map_err(|error| error.to_string())?;
        archive::extract(
            &archive_for_install,
            &staging_for_install,
            &manifest::load()?,
            &install_cancel,
        )
    })
    .await
    .inspect_err(|_| {
        let _ = remove_directory_if_exists(&staging);
        let _ = remove_file_if_exists(&archive_path);
    })?;
    if cancellation.is_cancelled() {
        remove_directory_if_exists(&staging)?;
        remove_file_if_exists(&archive_path)?;
        return Err(super::super::preparation::RESOURCE_PREPARATION_CANCELLED.to_string());
    }
    progress_update(
        progress,
        ResourcePreparationStage::Installing,
        manifest.archive_size,
        manifest.archive_size,
    );
    remove_directory_if_exists(&target)?;
    fs::rename(&staging, &target).map_err(|error| error.to_string())?;
    remove_file_if_exists(&archive_path)?;
    progress_update(
        progress,
        ResourcePreparationStage::Ready,
        manifest.archive_size,
        manifest.archive_size,
    );
    Ok(())
}

fn verify_directory(
    directory: &Path,
    manifest: &manifest::Manifest,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let mut actual = HashSet::with_capacity(manifest.files.len());
    for entry in walkdir::WalkDir::new(directory).follow_links(false) {
        let entry = entry.map_err(|error| error.to_string())?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
            return Err(format!(
                "Kokoro package contains a link: {}",
                entry.path().display()
            ));
        }
        if !metadata.is_file() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(directory)
            .map_err(|error| error.to_string())?;
        let relative = relative
            .to_str()
            .ok_or_else(|| "Kokoro package contains a non-UTF-8 path".to_string())?;
        actual.insert(relative.to_owned());
    }
    let expected = manifest
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<HashSet<_>>();
    if actual.len() != expected.len() || actual.iter().any(|path| !expected.contains(path.as_str()))
    {
        return Err("Kokoro package file set differs from its manifest".to_string());
    }
    for file in &manifest.files {
        let path = directory.join(&file.path);
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if !metadata.is_file() || metadata.len() != file.size || is_reparse_point(&metadata) {
            return Err(format!(
                "Kokoro package file metadata differs: {}",
                file.path
            ));
        }
        verify_file(&path, &file.sha256, cancellation)?;
    }
    Ok(())
}

fn verify_file(
    path: &Path,
    expected: &str,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        if cancellation.is_cancelled() {
            return Err(super::super::preparation::RESOURCE_PREPARATION_CANCELLED.to_string());
        }
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    if hex::encode(hash.finalize()) != expected {
        return Err(format!(
            "Kokoro package file failed SHA-256 verification: {}",
            path.display()
        ));
    }
    Ok(())
}

fn progress_update(
    progress: &mut impl FnMut(ResourcePreparationProgress),
    stage: ResourcePreparationStage,
    downloaded: u64,
    total: u64,
) {
    progress(ResourcePreparationProgress {
        stage,
        downloaded,
        total: Some(total),
        percent: Some(((downloaded.saturating_mul(100) / total).min(100)) as u8),
    });
}

fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn remove_directory_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

#[cfg(target_os = "windows")]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(target_os = "windows"))]
const fn is_reparse_point(_: &fs::Metadata) -> bool {
    false
}
