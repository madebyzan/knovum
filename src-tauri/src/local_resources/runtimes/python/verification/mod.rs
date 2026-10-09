use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

use super::{RuntimeError, MANIFEST_NAME};

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

pub(super) fn manifest_exists(path: &Path) -> Result<bool, RuntimeError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn read_manifest(path: &Path) -> Result<Vec<u8>, RuntimeError> {
    if path.file_name().and_then(|name| name.to_str()) != Some(MANIFEST_NAME) {
        return Err(RuntimeError::InvalidPublication(
            "unexpected manifest file name".into(),
        ));
    }
    let metadata = filesystem::checked_metadata(path)?;
    if !metadata.is_file() || metadata.len() > MAX_MANIFEST_BYTES {
        return Err(RuntimeError::InvalidPublication(
            "manifest is not a bounded regular file".into(),
        ));
    }
    fs::read(path).map_err(RuntimeError::from)
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
) -> Result<(PathBuf, PathBuf), RuntimeError> {
    let root = manifest_path.parent().ok_or_else(|| {
        RuntimeError::InvalidPublication("manifest has no parent directory".into())
    })?;
    let root_metadata = filesystem::checked_metadata(root)?;
    if !root_metadata.is_dir() {
        return Err(RuntimeError::InvalidPublication(
            "runtime root is not a directory".into(),
        ));
    }
    let manifest: RuntimeManifest = serde_json::from_slice(manifest_bytes)
        .map_err(|error| RuntimeError::InvalidPublication(error.to_string()))?;
    validate_identity(&manifest)?;
    if manifest.profile != profile {
        return Err(RuntimeError::InvalidPublication(
            "runtime profile does not match the requested capability".into(),
        ));
    }
    if let Some(descriptor) = descriptor {
        if manifest_digest != descriptor.manifest_sha256
            || manifest.dependency_requirements_sha256 != descriptor.dependency_requirements_sha256
            || manifest.profile != descriptor.profile
            || manifest.dependency_lock_sha256 != descriptor.dependency_lock_sha256
            || manifest.version != descriptor.version
        {
            return Err(RuntimeError::InvalidPublication(
                "runtime differs from the trusted descriptor".into(),
            ));
        }
    }
    #[cfg(debug_assertions)]
    if descriptor.is_none() && manifest.dependency_lock_sha256 != digest(profile.lock_bytes()) {
        return Err(RuntimeError::InvalidPublication(
            "development dependency lock differs from this checkout".into(),
        ));
    }
    if !manifest.files.iter().any(|file| {
        file.path == "runtime-requirements.txt"
            && file.sha256 == manifest.dependency_requirements_sha256
    }) {
        return Err(RuntimeError::InvalidPublication(
            "requirements identity differs from declared files".into(),
        ));
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

fn validate_identity(manifest: &RuntimeManifest) -> Result<(), RuntimeError> {
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
        return Err(RuntimeError::InvalidPublication(
            "manifest identity does not match this build".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(super) mod tests;
