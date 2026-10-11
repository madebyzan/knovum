use knovum_inference_contracts::runtime_manifest::RuntimeManifest;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};
#[cfg(not(debug_assertions))]
use tauri::Manager;
use tauri::{AppHandle, Runtime};

#[derive(Clone)]
pub(crate) struct InferenceRuntimePaths {
    pub(crate) manifest_path: PathBuf,
    pub(crate) executable: PathBuf,
    pub(crate) digest: String,
}

pub(crate) fn resolve<R: Runtime>(app: &AppHandle<R>) -> Result<InferenceRuntimePaths, String> {
    #[cfg(windows)]
    if windows_version::OsVersion::current() < windows_version::OsVersion::new(10, 0, 0, 19041) {
        return Err(
            "unsupported_device: Rust inference requires Windows 10 2004 (build 19041) or later"
                .into(),
        );
    }
    #[cfg(debug_assertions)]
    let (directory, executable) = {
        let _ = app;
        let directory = PathBuf::from(env!("KNOVUM_INFERENCE_RUNTIME_DIRECTORY"));
        let executable = std::env::current_exe()
            .map_err(|error| error.to_string())?
            .parent()
            .ok_or("runtime_missing: application executable has no parent")?
            .join(if cfg!(windows) {
                "knovum-inference-worker.exe"
            } else {
                "knovum-inference-worker"
            });
        (directory, executable)
    };
    #[cfg(not(debug_assertions))]
    let (directory, executable) = {
        let directory = app
            .path()
            .resource_dir()
            .map_err(|error| error.to_string())?
            .join("resources/inference-runtime");
        let executable = directory.join("bin").join(if cfg!(windows) {
            "knovum-inference-worker.exe"
        } else {
            "knovum-inference-worker"
        });
        (directory, executable)
    };
    let manifest_path = directory.join("inference-runtime-manifest.json");
    let bytes = read_manifest(&manifest_path)?;
    let runtime = InferenceRuntimePaths {
        manifest_path,
        executable,
        digest: hex::encode(Sha256::digest(&bytes)),
    };
    verify_before_spawn(&runtime)?;
    Ok(runtime)
}

pub(crate) fn verify_before_spawn(runtime: &InferenceRuntimePaths) -> Result<(), String> {
    let bytes = read_manifest(&runtime.manifest_path)?;
    if hex::encode(Sha256::digest(&bytes)) != runtime.digest {
        return Err("runtime_mismatch: inference manifest changed after resolution".into());
    }
    let actual: RuntimeManifest =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    let expected: RuntimeManifest = serde_json::from_slice(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/inference-runtime-manifest.json"
    )))
    .expect("Build embeds the trusted inference manifest");
    if actual.schema_version != expected.schema_version
        || actual.target != expected.target
        || actual.onnx_runtime != expected.onnx_runtime
        || actual.files.len() != expected.files.len()
        || actual
            .files
            .iter()
            .map(|file| &file.path)
            .collect::<std::collections::HashSet<_>>()
            .len()
            != actual.files.len()
        || actual
            .files
            .iter()
            .any(|file| !expected.files.contains(file))
    {
        return Err("runtime_mismatch: inference runtime supplier inventory differs".into());
    }
    let directory = runtime
        .manifest_path
        .parent()
        .ok_or("runtime_missing: inference directory is missing")?;
    crate::local_resources::filesystem::verify_regular_directory(directory)?;
    let mut files = 0;
    for entry in walkdir::WalkDir::new(directory)
        .min_depth(1)
        .follow_links(false)
    {
        let entry = entry.map_err(|error| format!("runtime_mismatch: {error}"))?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
        if metadata.is_dir() {
            crate::local_resources::filesystem::verify_regular_directory(entry.path())?;
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(directory)
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        if !metadata.is_file()
            || relative != "inference-runtime-manifest.json"
                && !expected.files.iter().any(|file| file.path == relative)
        {
            return Err("runtime_mismatch: inference runtime contains an unverified file".into());
        }
        files += 1;
        if files > 257 {
            return Err("runtime_mismatch: inference runtime inventory exceeds its bound".into());
        }
    }
    if files != expected.files.len() + 1 {
        return Err("runtime_mismatch: inference runtime inventory is incomplete".into());
    }
    for file in &expected.files {
        let path = directory.join(&file.path);
        let metadata =
            fs::symlink_metadata(&path).map_err(|error| format!("runtime_missing: {error}"))?;
        if !metadata.is_file()
            || metadata.len() != file.size
            || !crate::local_resources::verification::verify_cancellable(
                &path,
                &file.sha256,
                &tokio_util::sync::CancellationToken::new(),
                "cancelled",
            )?
        {
            return Err("runtime_mismatch: inference runtime library differs".into());
        }
    }
    if !fs::symlink_metadata(&runtime.executable)
        .map_err(|error| format!("runtime_missing: {error}"))?
        .is_file()
    {
        return Err("runtime_missing: inference worker is not a regular file".into());
    }
    Ok(())
}

fn read_manifest(path: &std::path::Path) -> Result<Vec<u8>, String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| format!("runtime_missing: {error}"))?;
    if !metadata.is_file() || metadata.len() > 1_048_576 {
        return Err("runtime_mismatch: inference manifest must be a bounded regular file".into());
    }
    fs::read(path).map_err(|error| error.to_string())
}
