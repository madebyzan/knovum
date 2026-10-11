use std::path::{Component, Path};

use tokio_util::sync::CancellationToken;
use zip::{CompressionMethod, ZipArchive};

use super::{control, PACKAGE_VERSION};

pub const MAX_ENTRIES: usize = 50_000;
pub const MAX_MANIFEST: u64 = 32 * 1024 * 1024;
pub const MAX_UNPACKED: u64 = 20 * 1024 * 1024 * 1024;
pub const MAX_ASSETS: usize = 10_000;

pub fn validate_token(token: &str) -> Result<(), String> {
    if token.is_empty()
        || token.len() > 64
        || !token
            .bytes()
            .all(|value| value.is_ascii_alphanumeric() || value == b'-' || value == b'_')
    {
        return Err("Space package asset token is invalid".into());
    }
    Ok(())
}

pub fn validate_basename(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || name.contains(':')
    {
        return Err("Space package asset name is invalid".into());
    }
    Ok(())
}

pub fn validate_entry_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.starts_with('/') || name.contains('\\') || name.contains(':') {
        return Err("Space package contains an unsafe entry path".into());
    }
    let trimmed = name.strip_suffix('/').unwrap_or(name);
    if trimmed.is_empty()
        || trimmed
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err("Space package contains an unsafe entry path".into());
    }
    if name != "manifest.json" && name != "assets-index.json" && !name.starts_with("assets/") {
        return Err("Space package contains an unknown entry".into());
    }
    Ok(())
}

pub fn validate_relative_path(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("Package asset entry is incomplete".into());
    }
    for component in path.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err("Space package contains an unsafe asset path".into());
        }
    }
    Ok(())
}

pub fn validate_manifest_version(manifest: &str) -> Result<(), String> {
    let value: serde_json::Value = serde_json::from_str(manifest)
        .map_err(|error| format!("Space package manifest is invalid JSON: {error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "Space package manifest must be an object".to_string())?;
    if object.get("format").and_then(serde_json::Value::as_str) != Some("knovum-space") {
        return Err("Unsupported Space package format".into());
    }
    let version = object
        .get("version")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "Space package version is required".to_string())?;
    if version != PACKAGE_VERSION {
        return Err(format!(
            "Unsupported Space package version; this build accepts v{PACKAGE_VERSION}"
        ));
    }
    Ok(())
}

pub fn validate_archive(
    archive: &mut ZipArchive<std::fs::File>,
    cancellation: &CancellationToken,
) -> Result<u64, crate::foundation::app_error::AppError> {
    use std::collections::HashSet;
    if archive.len() > MAX_ENTRIES {
        return Err("Space package contains too many entries".into());
    }
    let mut names = HashSet::new();
    let mut total = 0_u64;
    for index in 0..archive.len() {
        control::check(cancellation)?;
        let file = archive
            .by_index_raw(index)
            .map_err(|error| error.to_string())?;
        if file.compression() != CompressionMethod::Stored {
            return Err("Space package entries must use Stored compression".into());
        }
        validate_entry_name(file.name())?;
        if !names.insert(file.name().to_owned()) {
            return Err("Space package contains duplicate entries".into());
        }
        validate_unix_mode(file.unix_mode())?;
        total = total
            .checked_add(file.size())
            .ok_or("Space package size overflow")?;
        if total > MAX_UNPACKED {
            return Err("Space package exceeds the unpacked size limit".into());
        }
    }
    Ok(total)
}

fn validate_unix_mode(mode: Option<u32>) -> Result<(), String> {
    let Some(mode) = mode else {
        return Ok(());
    };
    let kind = mode & 0o170000;
    if kind == 0 || kind == 0o100000 || kind == 0o040000 {
        return Ok(());
    }
    Err("Space package contains a symbolic link or special file".into())
}

#[cfg(test)]
mod tests;
