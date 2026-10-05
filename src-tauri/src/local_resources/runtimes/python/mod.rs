use std::{
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use tauri::Manager;
use tauri::{AppHandle, Runtime};

mod archive;
mod coordinator;
mod descriptor;
mod installation;
mod retention;
pub(crate) use coordinator::prepare;
pub(crate) use descriptor::RuntimeStatus;

mod verification;

const MANIFEST_NAME: &str = "runtime-manifest.json";
#[cfg(debug_assertions)]
const DEVELOPMENT_MANIFEST_KEY: &str = "KNOVUM_PYTHON_RUNTIME_MANIFEST_PATH";

#[derive(Clone)]
pub(crate) struct PythonRuntimePaths {
    pub(crate) manifest_path: PathBuf,
    pub(crate) executable: PathBuf,
    pub(crate) package_root: PathBuf,
    pub(crate) digest: String,
    pub(crate) worker_root: PathBuf,
    pub(crate) lease: Option<std::sync::Arc<std::fs::File>>,
}

pub(crate) fn verify_before_spawn(runtime: &PythonRuntimePaths) -> Result<(), String> {
    let bytes = verification::read_manifest(&runtime.manifest_path)?;
    let digest = verification::digest(&bytes);
    if digest != runtime.digest {
        return Err("python-runtime-invalid: runtime manifest changed after resolution".into());
    }
    let current = verification::verify_runtime(
        &runtime.manifest_path,
        &bytes,
        digest,
        &tokio_util::sync::CancellationToken::new(),
        descriptor::embedded()?.as_ref(),
    )?;
    if current.0 != runtime.executable || current.1 != runtime.package_root {
        return Err("python-runtime-invalid: runtime paths changed after resolution".into());
    }
    Ok(())
}

struct CachedRuntime {
    manifest_path: PathBuf,
    paths: PythonRuntimePaths,
}

static VERIFIED_RUNTIME: OnceLock<Mutex<Option<CachedRuntime>>> = OnceLock::new();

pub(crate) fn resolve_python_runtime<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<Option<PythonRuntimePaths>, String> {
    let manifest_path = match configured_manifest()? {
        Some(path) => path,
        None => match descriptor::embedded()? {
            Some(value) => descriptor::destination(app, &value)?.join(MANIFEST_NAME),
            None => return Ok(None),
        },
    };
    let lease = if configured_manifest()?.is_some() {
        None
    } else {
        if !manifest_path.is_file() {
            return Ok(None);
        }
        Some(retention::acquire(
            manifest_path
                .parent()
                .ok_or("Runtime manifest has no parent")?,
        )?)
    };
    if !manifest_path.is_file() {
        return Ok(None);
    }

    let cache = VERIFIED_RUNTIME.get_or_init(|| Mutex::new(None));
    let mut cached = cache
        .lock()
        .map_err(|_| "python-runtime-invalid: verification cache is poisoned".to_string())?;
    #[cfg(not(debug_assertions))]
    if let Some(runtime) = cached
        .as_ref()
        .filter(|runtime| runtime.manifest_path == manifest_path)
    {
        return Ok(Some(runtime.paths.clone()));
    }
    #[cfg(not(debug_assertions))]
    cached.take();

    let manifest_bytes = verification::read_manifest(&manifest_path)?;
    let digest = verification::digest(&manifest_bytes);
    #[cfg(debug_assertions)]
    if let Some(runtime) = cached
        .as_ref()
        .filter(|runtime| runtime.manifest_path == manifest_path && runtime.paths.digest == digest)
    {
        return Ok(Some(runtime.paths.clone()));
    }
    #[cfg(debug_assertions)]
    cached.take();

    let (executable, package_root) = verification::verify_runtime(
        &manifest_path,
        &manifest_bytes,
        digest.clone(),
        &tokio_util::sync::CancellationToken::new(),
        descriptor::embedded()?.as_ref(),
    )?;
    let paths = PythonRuntimePaths {
        manifest_path: manifest_path.clone(),
        executable,
        package_root,
        digest,
        worker_root: worker_root(app)?,
        lease,
    };
    *cached = Some(CachedRuntime {
        manifest_path,
        paths: paths.clone(),
    });
    Ok(Some(paths))
}

#[cfg(debug_assertions)]
fn configured_manifest() -> Result<Option<PathBuf>, String> {
    super::super::configured_path::configured_path(DEVELOPMENT_MANIFEST_KEY)
}

#[cfg(not(debug_assertions))]
fn configured_manifest() -> Result<Option<PathBuf>, String> {
    Ok(None)
}

pub(crate) fn status<R: Runtime>(app: &AppHandle<R>) -> Result<RuntimeStatus, String> {
    let value = descriptor::embedded()?;
    Ok(RuntimeStatus {
        available: resolve_python_runtime(app)?.is_some(),
        downloadable: value.is_some(),
        download_size: value.map(|value| value.size),
    })
}

fn worker_root<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    #[cfg(debug_assertions)]
    if let Some(root) =
        crate::local_resources::configured_path::configured_path("KNOVUM_PYTHON_WORKER_PATH")?
    {
        return checked_worker_root(root);
    }
    let root = app
        .path()
        .resource_dir()
        .map_err(|error| error.to_string())?
        .join("resources/python-worker");
    checked_worker_root(root)
}

fn checked_worker_root(root: PathBuf) -> Result<PathBuf, String> {
    if !root.join("knovum_inference/server.py").is_file() {
        return Err("runtime_missing: application Python Worker is missing".into());
    }
    Ok(root)
}

pub(crate) fn required_digest<R: Runtime>(app: &AppHandle<R>) -> Result<String, String> {
    #[cfg(debug_assertions)]
    if configured_manifest()?.is_some() {
        return resolve_python_runtime(app)?
            .map(|runtime| runtime.digest)
            .ok_or_else(|| "runtime_missing: configured development runtime is missing".into());
    }
    let _ = app;
    Ok(descriptor::required()?.manifest_sha256)
}

pub(super) fn retire_unused<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    if let Some(value) = descriptor::embedded()? {
        retention::retire_unused(&descriptor::destination(app, &value)?)?;
    }
    Ok(())
}
