use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

use super::MANIFEST_NAME;

mod filesystem;

const MAX_MANIFEST_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RuntimeManifest {
    schema_version: u8,
    runtime_id: String,
    version: String,
    dependency_requirements_sha256: String,
    profile: super::RuntimeProfile,
    dependency_lock_sha256: String,
    target: String,
    executable: String,
    package_root: String,
    files: Vec<RuntimeFile>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RuntimeFile {
    pub(super) path: String,
    pub(super) size: u64,
    pub(super) sha256: String,
}

pub(super) fn read_manifest(path: &Path) -> Result<Vec<u8>, String> {
    if path.file_name().and_then(|name| name.to_str()) != Some(MANIFEST_NAME) {
        return Err("python-runtime-invalid: unexpected manifest file name".to_string());
    }
    let metadata = filesystem::checked_metadata(path)?;
    if !metadata.is_file() || metadata.len() > MAX_MANIFEST_BYTES {
        return Err("python-runtime-invalid: manifest is not a bounded regular file".to_string());
    }
    fs::read(path).map_err(|error| error.to_string())
}

pub(super) fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub(super) fn verify_runtime(
    manifest_path: &Path,
    manifest_bytes: &[u8],
    manifest_digest: String,
    cancellation: &CancellationToken,
    descriptor: Option<&super::descriptor::Descriptor>,
    profile: super::RuntimeProfile,
) -> Result<(PathBuf, PathBuf), String> {
    let root = manifest_path
        .parent()
        .ok_or_else(|| "python-runtime-invalid: manifest has no parent directory".to_string())?;
    let root_metadata = filesystem::checked_metadata(root)?;
    if !root_metadata.is_dir() {
        return Err("python-runtime-invalid: runtime root is not a directory".to_string());
    }
    let manifest: RuntimeManifest = serde_json::from_slice(manifest_bytes)
        .map_err(|error| format!("python-runtime-invalid: {error}"))?;
    validate_identity(&manifest)?;
    if manifest.profile != profile {
        return Err(
            "python-runtime-invalid: runtime profile does not match the requested capability"
                .into(),
        );
    }
    if let Some(descriptor) = descriptor {
        if manifest_digest != descriptor.manifest_sha256
            || manifest.dependency_requirements_sha256 != descriptor.dependency_requirements_sha256
            || manifest.profile != descriptor.profile
            || manifest.dependency_lock_sha256 != descriptor.dependency_lock_sha256
            || manifest.version != descriptor.version
        {
            return Err(
                "python-runtime-invalid: runtime differs from the trusted descriptor".into(),
            );
        }
    }
    #[cfg(debug_assertions)]
    if descriptor.is_none() && manifest.dependency_lock_sha256 != digest(profile.lock_bytes()) {
        return Err(
            "python-runtime-invalid: development dependency lock differs from this checkout".into(),
        );
    }
    if !manifest.files.iter().any(|file| {
        file.path == "runtime-requirements.txt"
            && file.sha256 == manifest.dependency_requirements_sha256
    }) {
        return Err(
            "python-runtime-invalid: requirements identity differs from declared files".into(),
        );
    }
    let (executable, package_root) = filesystem::verify(
        root,
        &manifest.files,
        &manifest.executable,
        &manifest.package_root,
        cancellation,
    )?;
    Ok((executable, package_root))
}

fn validate_identity(manifest: &RuntimeManifest) -> Result<(), String> {
    let executable = if cfg!(target_os = "windows") {
        "python.exe"
    } else {
        "bin/python3"
    };
    if manifest.schema_version != 1
        || manifest.runtime_id != "cpython"
        || manifest.version != "3.11.17"
        || manifest.dependency_requirements_sha256.len() != 64
        || manifest.dependency_lock_sha256.len() != 64
        || manifest.target != crate::local_resources::tools::platform_directory()
        || manifest.executable != executable
        || manifest.package_root != "packages"
    {
        return Err(
            "python-runtime-invalid: manifest identity does not match this build".to_string(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests;
