use std::{fs, path::Path};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::{PythonRuntimePaths, MANIFEST_NAME};

mod filesystem;

const MAX_MANIFEST_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RuntimeManifest {
    schema_version: u8,
    runtime_id: String,
    version: String,
    protocol_version: u32,
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
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn verify_runtime(
    manifest_path: &Path,
    manifest_bytes: &[u8],
    manifest_digest: String,
) -> Result<PythonRuntimePaths, String> {
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
    let (executable, package_root) = filesystem::verify(
        root,
        &manifest.files,
        &manifest.executable,
        &manifest.package_root,
    )?;
    Ok(PythonRuntimePaths {
        manifest_path: manifest_path.to_owned(),
        executable,
        package_root,
        digest: manifest_digest,
    })
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
        || manifest.protocol_version != 1
        || manifest.dependency_lock_sha256
            != digest(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../python/uv.lock"
            )))
        || manifest.target != super::super::platform_directory()
        || manifest.executable != executable
        || manifest.package_root != "packages"
    {
        return Err(
            "python-runtime-invalid: manifest identity does not match this build".to_string(),
        );
    }
    Ok(())
}
