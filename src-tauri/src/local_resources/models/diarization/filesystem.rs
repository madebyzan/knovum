use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, Runtime};

use super::manifest;

pub(super) fn copy_verified(
    source: &Path,
    target: &Path,
    expected: &manifest::File,
) -> Result<(), String> {
    let parent = source
        .parent()
        .ok_or("Diarization source file has no parent")?;
    let parent_metadata = fs::symlink_metadata(parent).map_err(|error| error.to_string())?;
    if !parent_metadata.is_dir()
        || parent_metadata.file_type().is_symlink()
        || is_reparse_point(&parent_metadata)
    {
        return Err(format!(
            "Diarization model directory is linked: {}",
            expected.path
        ));
    }
    let metadata = fs::symlink_metadata(source)
        .map_err(|_| format!("Missing diarization model file {}", expected.path))?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || is_reparse_point(&metadata)
        || metadata.len() != expected.size
    {
        return Err(format!(
            "Diarization model file metadata differs: {}",
            expected.path
        ));
    }
    verify_file(source, expected.size, expected.sha256)?;
    fs::create_dir_all(
        target
            .parent()
            .ok_or("Diarization model file has no parent")?,
    )
    .map_err(|error| error.to_string())?;
    let mut input = fs::File::open(source).map_err(|error| error.to_string())?;
    let mut output = fs::File::create(target).map_err(|error| error.to_string())?;
    let mut hash = Sha256::new();
    let mut copied = 0_u64;
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = input.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        copied += read as u64;
        if copied > expected.size {
            return Err(format!(
                "Diarization model file grew during import: {}",
                expected.path
            ));
        }
        hash.update(&buffer[..read]);
        output
            .write_all(&buffer[..read])
            .map_err(|error| error.to_string())?;
    }
    output.sync_all().map_err(|error| error.to_string())?;
    if copied != expected.size || hex::encode(hash.finalize()) != expected.sha256 {
        return Err(format!(
            "Diarization model file failed verification: {}",
            expected.path
        ));
    }
    Ok(())
}

pub(super) fn verify_directory(directory: &Path) -> Result<(), String> {
    verify_regular_directory(directory)?;
    let canonical_root = directory
        .canonicalize()
        .map_err(|error| error.to_string())?;
    for child in ["embedding", "segmentation", "plda"] {
        let path = directory.join(child);
        verify_regular_directory(&path)?;
        if !path
            .canonicalize()
            .map_err(|error| error.to_string())?
            .starts_with(&canonical_root)
        {
            return Err(format!(
                "Diarization model directory escapes its root: {}",
                path.display()
            ));
        }
    }
    let config = directory.join("config.yaml");
    verify_bounded_file(
        &config,
        &canonical_root,
        manifest::CONFIG.len() as u64,
        manifest::CONFIG_SHA256,
    )?;
    for file in manifest::FILES {
        verify_bounded_file(
            &directory.join(file.path),
            &canonical_root,
            file.size,
            file.sha256,
        )?;
    }
    Ok(())
}

fn verify_bounded_file(path: &Path, root: &Path, size: u64, sha256: &str) -> Result<(), String> {
    if !path
        .canonicalize()
        .map_err(|error| error.to_string())?
        .starts_with(root)
    {
        return Err(format!(
            "Diarization model file escapes its root: {}",
            path.display()
        ));
    }
    verify_file(path, size, sha256)
}

fn verify_regular_directory(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        return Err(format!(
            "Diarization model directory is linked: {}",
            path.display()
        ));
    }
    Ok(())
}

pub(super) fn verify_managed_root<R: Runtime>(
    app: &AppHandle<R>,
    root: &Path,
) -> Result<(), String> {
    verify_regular_directory(root)?;
    let local_resources = root
        .parent()
        .ok_or("Local resource directory is unavailable")?;
    verify_regular_directory(local_resources)?;
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    let canonical_app_data = app_data.canonicalize().map_err(|error| error.to_string())?;
    let canonical_local_resources = local_resources
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let canonical_root = root.canonicalize().map_err(|error| error.to_string())?;
    if !canonical_root.starts_with(&canonical_app_data)
        || canonical_root.parent() != Some(canonical_local_resources.as_path())
    {
        return Err("Diarization model directory escapes application data".into());
    }
    Ok(())
}

fn verify_file(path: &Path, size: u64, sha256: &str) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || is_reparse_point(&metadata)
        || metadata.len() != size
    {
        return Err(format!(
            "Diarization model file metadata differs: {}",
            path.display()
        ));
    }
    let mut input = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut hash = Sha256::new();
    let mut read_bytes = 0_u64;
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = input.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        read_bytes = read_bytes
            .checked_add(read as u64)
            .ok_or("Diarization model size exceeds its supported range")?;
        hash.update(&buffer[..read]);
    }
    if read_bytes != size || hex::encode(hash.finalize()) != sha256 {
        return Err(format!(
            "Diarization model digest differs: {}",
            path.display()
        ));
    }
    Ok(())
}

fn remove_directory(path: &Path) -> Result<(), String> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

pub(super) fn remove_owned_directory(path: &Path, parent: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
        Ok(metadata) => {
            if path.parent() != Some(parent) {
                return Err("Diarization staging directory is outside its managed root".into());
            }
            if metadata.file_type().is_symlink() {
                return remove_link(path);
            }
            if is_reparse_point(&metadata) {
                return if metadata.is_dir() {
                    fs::remove_dir(path).map_err(|error| error.to_string())
                } else {
                    fs::remove_file(path).map_err(|error| error.to_string())
                };
            }
            if !metadata.is_dir() {
                return Err("Diarization managed path is not a directory".into());
            }
            remove_directory(path)
        }
    }
}

#[cfg(target_os = "windows")]
fn remove_link(path: &Path) -> Result<(), String> {
    fs::remove_dir(path)
        .or_else(|_| fs::remove_file(path))
        .map_err(|error| error.to_string())
}

#[cfg(not(target_os = "windows"))]
fn remove_link(path: &Path) -> Result<(), String> {
    fs::remove_file(path).map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
pub(super) fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}
#[cfg(not(target_os = "windows"))]
pub(super) const fn is_reparse_point(_: &fs::Metadata) -> bool {
    false
}
