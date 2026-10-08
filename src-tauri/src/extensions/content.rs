use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};
use tauri::{AppHandle, Manager};
use unicode_normalization::UnicodeNormalization;

pub(super) const MAX_FILES: usize = 4096;
pub(super) const MAX_FILE_BYTES: usize = 16 * 1024 * 1024;
pub(super) const MAX_PACKAGE_BYTES: usize = 128 * 1024 * 1024;
pub(super) type PackageFiles = BTreeMap<String, Vec<u8>>;

pub(super) fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub(super) fn files_digest(files: &PackageFiles) -> String {
    let mut hasher = Sha256::new();
    for (path, bytes) in files {
        hasher.update((path.len() as u64).to_le_bytes());
        hasher.update(path.as_bytes());
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    }
    hex::encode(hasher.finalize())
}

pub(super) fn validate_path(path: &str) -> Result<(), String> {
    let invalid = path.is_empty()
        || path.contains(['\\', ':', '<', '>', '"', '|', '?', '*'])
        || path.chars().any(char::is_control)
        || Path::new(path)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        || path
            .split('/')
            .any(|part| part.is_empty() || part.ends_with(['.', ' ']) || windows_reserved(part));
    if invalid {
        Err(format!("package-path-invalid: {path}"))
    } else {
        Ok(())
    }
}

pub(in crate::extensions) fn filesystem_identity(path: &str) -> String {
    path.nfc().collect::<String>().to_lowercase()
}

fn windows_reserved(part: &str) -> bool {
    let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

pub(super) fn validate_files(files: &PackageFiles) -> Result<(), String> {
    if files.len() > MAX_FILES || files.values().map(Vec::len).sum::<usize>() > MAX_PACKAGE_BYTES {
        return Err("package-size-limit-exceeded".into());
    }
    for (path, bytes) in files {
        validate_path(path)?;
        if bytes.len() > MAX_FILE_BYTES {
            return Err(format!("package-file-size-limit: {path}"));
        }
    }
    validate_paths(files.keys().map(|path| (path.as_str(), false)))?;

    Ok(())
}

pub(in crate::extensions) fn validate_paths<'a>(
    paths: impl Iterator<Item = (&'a str, bool)>,
) -> Result<(), String> {
    let mut identities = BTreeMap::<String, (String, bool)>::new();
    for (path, is_directory) in paths {
        validate_path(path)?;
        let parts = path.split('/').collect::<Vec<_>>();
        for end in 1..=parts.len() {
            let prefix = parts[..end].join("/");
            let directory = end != parts.len() || is_directory;
            let identity = filesystem_identity(&prefix);
            if let Some((previous, previous_directory)) = identities.get(&identity) {
                if previous != &prefix || *previous_directory != directory {
                    return Err(format!("package-path-collision: {path}"));
                }
            } else {
                identities.insert(identity, (prefix, directory));
            }
        }
    }
    Ok(())
}

pub(super) fn root(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("agent-extensions"))
}

pub(super) fn publish(directory: &Path, files: &PackageFiles) -> Result<(), String> {
    validate_files(files)?;
    if directory.exists() {
        return verify(directory, files);
    }
    let parent = directory.parent().ok_or("content-root-invalid")?;
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let staging = parent.join(format!(".staging-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&staging).map_err(|error| error.to_string())?;
    let content_update = (|| {
        for (path, bytes) in files {
            let destination = staging.join(path);
            std::fs::create_dir_all(destination.parent().expect("content parent"))
                .map_err(|error| error.to_string())?;
            std::fs::write(destination, bytes).map_err(|error| error.to_string())?;
        }
        std::fs::rename(&staging, directory).map_err(|error| error.to_string())
    })();
    if content_update.is_err() {
        let _ = std::fs::remove_dir_all(staging);
    }
    content_update
}

fn verify(directory: &Path, files: &PackageFiles) -> Result<(), String> {
    let actual = super::plugins::sources::directory_files(
        directory,
        &tokio_util::sync::CancellationToken::new(),
    )
    .map_err(|error| error.message)?;
    if files_digest(&actual) != files_digest(files) {
        return Err("extension-content-corrupt".into());
    }
    Ok(())
}
