use knovum_inference_contracts::runtime_manifest::{RuntimeFile, RuntimeManifest};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
#[cfg(not(debug_assertions))]
use tauri::Manager;
use tauri::{AppHandle, Runtime};

/// The image the host spawns.
///
/// A worker cannot verify the bytes it is already executing, so the host owns this one check. A
/// packaged build executes a pinned inventory member; a development build executes this
/// repository's Cargo artifact, which no inventory pins.
#[derive(Clone)]
pub(crate) enum SpawnedImage {
    Pinned(RuntimeFile),
    DeveloperBuild,
}

#[derive(Clone)]
pub(crate) struct InferenceRuntimePaths {
    pub(crate) manifest_path: PathBuf,
    pub(crate) executable: PathBuf,
    pub(crate) digest: String,
    pub(crate) spawned_image: SpawnedImage,
    /// Bytes the worker hashes before it loads a library, which bounds its initialization.
    pub(crate) library_bytes: u64,
}

impl InferenceRuntimePaths {
    /// Bytes the host hashes before it spawns the worker, which bounds [`verify_before_spawn`].
    pub(crate) fn spawn_verified_bytes(&self) -> u64 {
        match &self.spawned_image {
            SpawnedImage::Pinned(image) => image.size,
            SpawnedImage::DeveloperBuild => 0,
        }
    }
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
    let manifest = supplied_manifest(&bytes)?;
    Ok(InferenceRuntimePaths {
        spawned_image: spawned_image(&manifest, &directory, &executable)?,
        library_bytes: manifest.files.iter().map(|file| file.size).sum(),
        manifest_path,
        executable,
        digest: hex::encode(Sha256::digest(&bytes)),
    })
}

/// Classifies the image this build executes. An image inside the runtime directory must be an
/// inventory member; one outside it belongs to a development build of this repository.
fn spawned_image(
    manifest: &RuntimeManifest,
    directory: &Path,
    executable: &Path,
) -> Result<SpawnedImage, String> {
    let Ok(relative) = executable.strip_prefix(directory) else {
        return Ok(SpawnedImage::DeveloperBuild);
    };
    let relative = relative
        .to_str()
        .ok_or("runtime_mismatch: inference worker path is invalid")?
        .replace('\\', "/");
    manifest
        .files
        .iter()
        .find(|file| file.path == relative)
        .cloned()
        .map(SpawnedImage::Pinned)
        .ok_or_else(|| "runtime_mismatch: inference worker is not an inventory member".to_owned())
}

/// The supplied inventory must be the one this build was qualified against. Hashing a library
/// belongs to the worker that loads it, so resolving a path stays cheap enough for every caller
/// that only needs its identity.
fn supplied_manifest(bytes: &[u8]) -> Result<RuntimeManifest, String> {
    let actual: RuntimeManifest =
        serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
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
    Ok(actual)
}

/// Establishes that the host is about to execute the qualified image from the inventory it
/// resolved.
///
/// Hashing a library belongs to the worker that loads it: that is the only process whose check
/// cannot be invalidated between the verification and the load, and the only place the cost is
/// unavoidable. The host verifies the one thing a worker cannot verify about itself.
pub(crate) fn verify_before_spawn(runtime: &InferenceRuntimePaths) -> Result<(), String> {
    let bytes = read_manifest(&runtime.manifest_path)?;
    if hex::encode(Sha256::digest(&bytes)) != runtime.digest {
        return Err("runtime_mismatch: inference manifest changed after resolution".into());
    }
    supplied_manifest(&bytes)?;
    let metadata = fs::symlink_metadata(&runtime.executable)
        .map_err(|error| format!("runtime_missing: {error}"))?;
    if !metadata.is_file() {
        return Err("runtime_missing: inference worker is not a regular file".into());
    }
    let SpawnedImage::Pinned(image) = &runtime.spawned_image else {
        return Ok(());
    };
    if metadata.len() != image.size
        || !crate::local_resources::verification::verify_cancellable(
            &runtime.executable,
            &image.sha256,
            &tokio_util::sync::CancellationToken::new(),
            "cancelled",
        )?
    {
        return Err("runtime_mismatch: inference worker differs".into());
    }
    Ok(())
}

fn read_manifest(path: &Path) -> Result<Vec<u8>, String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| format!("runtime_missing: {error}"))?;
    if !metadata.is_file() || metadata.len() > 1_048_576 {
        return Err("runtime_mismatch: inference manifest must be a bounded regular file".into());
    }
    fs::read(path).map_err(|error| error.to_string())
}
