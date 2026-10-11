use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

#[cfg(not(debug_assertions))]
use tauri::Manager;
use tauri::{AppHandle, Runtime};

mod archive;
mod coordinator;
mod descriptor;
#[cfg(debug_assertions)]
mod development;
mod error;
#[cfg(not(debug_assertions))]
mod installation;
mod profile;
use super::retention;
pub(crate) use coordinator::prepare;
pub(crate) use descriptor::RuntimeStatus;
pub(crate) use error::RuntimeError;
pub(crate) use profile::RuntimeProfile;
pub(crate) use retention::RuntimeLease;

#[cfg(test)]
mod tests;
mod verification;

const MANIFEST_NAME: &str = "runtime-manifest.json";

#[derive(Clone)]
pub(crate) struct PythonRuntimePaths {
    pub(crate) profile: RuntimeProfile,
    pub(crate) manifest_path: PathBuf,
    pub(crate) executable: PathBuf,
    pub(crate) package_root: PathBuf,
    pub(crate) digest: String,
    pub(crate) worker_root: PathBuf,
    pub(crate) lease: Option<std::sync::Arc<RuntimeLease>>,
}

pub(crate) fn verify_before_spawn(runtime: &PythonRuntimePaths) -> Result<(), RuntimeError> {
    let bytes = verification::read_manifest(&runtime.manifest_path)?;
    let digest = verification::digest(&bytes);
    if digest != runtime.digest {
        return Err(RuntimeError::InvalidPublication(
            "runtime manifest changed after resolution".into(),
        ));
    }
    let current = verification::verify_runtime(
        &runtime.manifest_path,
        &bytes,
        digest,
        &tokio_util::sync::CancellationToken::new(),
        verification_descriptor(runtime.profile)?.as_ref(),
        runtime.profile,
    )?;
    if current.0 != runtime.executable || current.1 != runtime.package_root {
        return Err(RuntimeError::InvalidPublication(
            "runtime paths changed after resolution".into(),
        ));
    }
    Ok(())
}

static VERIFIED_RUNTIMES: OnceLock<Mutex<HashMap<RuntimeProfile, PythonRuntimePaths>>> =
    OnceLock::new();

pub(crate) fn resolve<R: Runtime>(
    app: &AppHandle<R>,
    profile: RuntimeProfile,
) -> Result<Option<PythonRuntimePaths>, RuntimeError> {
    #[cfg(debug_assertions)]
    let manifest_path = profile.development_manifest();
    #[cfg(not(debug_assertions))]
    let manifest_path = match descriptor::embedded(profile)? {
        Some(value) => descriptor::destination(app, &value)?.join(MANIFEST_NAME),
        None => return Ok(None),
    };
    if !verification::manifest_exists(&manifest_path)? {
        return Ok(None);
    }
    let lease = retention::acquire(&identity_directory(&manifest_path)?)?;
    if !verification::manifest_exists(&manifest_path)? {
        return Ok(None);
    }

    resolve_with_lease(app, profile, manifest_path, lease).map(Some)
}

fn resolve_with_lease<R: Runtime>(
    app: &AppHandle<R>,
    profile: RuntimeProfile,
    manifest_path: PathBuf,
    lease: std::sync::Arc<RuntimeLease>,
) -> Result<PythonRuntimePaths, RuntimeError> {
    let cache = VERIFIED_RUNTIMES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cached = cache.lock().map_err(|_| {
        RuntimeError::Infrastructure("Runtime verification cache is poisoned".into())
    })?;
    #[cfg(not(debug_assertions))]
    if let Some(runtime) = cached
        .get(&profile)
        .filter(|runtime| runtime.manifest_path == manifest_path)
    {
        return Ok(runtime.clone());
    }
    #[cfg(not(debug_assertions))]
    cached.remove(&profile);

    let manifest_bytes = verification::read_manifest(&manifest_path)?;
    let digest = verification::digest(&manifest_bytes);
    #[cfg(debug_assertions)]
    if let Some(runtime) = cached
        .get(&profile)
        .filter(|runtime| runtime.manifest_path == manifest_path && runtime.digest == digest)
    {
        return Ok(runtime.clone());
    }
    #[cfg(debug_assertions)]
    cached.remove(&profile);

    let (executable, package_root) = verification::verify_runtime(
        &manifest_path,
        &manifest_bytes,
        digest.clone(),
        &tokio_util::sync::CancellationToken::new(),
        verification_descriptor(profile)?.as_ref(),
        profile,
    )?;
    let paths = PythonRuntimePaths {
        profile,
        manifest_path: manifest_path.clone(),
        executable,
        package_root,
        digest,
        worker_root: worker_root(app, profile)?,
        lease: Some(lease),
    };
    cached.insert(profile, paths.clone());
    Ok(paths)
}

pub(crate) fn status<R: Runtime>(app: &AppHandle<R>, profile: RuntimeProfile) -> RuntimeStatus {
    let inspect = || -> Result<RuntimeStatus, String> {
        if resolve(app, profile)?.is_some() {
            return Ok(RuntimeStatus::Ready);
        }
        Ok(match verification_descriptor(profile)? {
            Some(value) => RuntimeStatus::NotInstalled {
                download_size: value.size,
            },
            None => RuntimeStatus::NotPublished,
        })
    };
    inspect().unwrap_or_else(|message| RuntimeStatus::Invalid { message })
}

fn verification_descriptor(
    profile: RuntimeProfile,
) -> Result<Option<descriptor::Descriptor>, RuntimeError> {
    #[cfg(debug_assertions)]
    {
        let _ = profile;
        Ok(None)
    }
    #[cfg(not(debug_assertions))]
    descriptor::embedded(profile)
}

#[cfg(debug_assertions)]
fn worker_root<R: Runtime>(_: &AppHandle<R>, profile: RuntimeProfile) -> Result<PathBuf, String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../python/src");
    checked_worker_root(root, profile)
}

#[cfg(not(debug_assertions))]
fn worker_root<R: Runtime>(app: &AppHandle<R>, profile: RuntimeProfile) -> Result<PathBuf, String> {
    let root = app
        .path()
        .resource_dir()
        .map_err(|error| error.to_string())?
        .join("resources/python-worker");
    checked_worker_root(root, profile)
}

fn checked_worker_root(root: PathBuf, profile: RuntimeProfile) -> Result<PathBuf, String> {
    if !root.join(profile.entry()).is_file() {
        return Err(format!(
            "runtime_missing: application Python Worker entry {} is missing in {}",
            profile.entry(),
            root.display()
        ));
    }
    Ok(root)
}

pub(super) fn retire_unused<R: Runtime>(
    app: &AppHandle<R>,
    profile: RuntimeProfile,
) -> Result<(), String> {
    #[cfg(debug_assertions)]
    {
        let _ = app;
        retention::retire_unused(&identity_directory(&profile.development_manifest())?)?;
    }
    #[cfg(not(debug_assertions))]
    if let Some(value) = verification_descriptor(profile)? {
        retention::retire_unused(&descriptor::destination(app, &value)?)?;
    }
    Ok(())
}

fn identity_directory(manifest: &std::path::Path) -> Result<PathBuf, String> {
    let runtime = manifest.parent().ok_or("Runtime manifest has no parent")?;
    #[cfg(debug_assertions)]
    let runtime = runtime
        .parent()
        .ok_or("Development runtime has no identity directory")?;
    Ok(runtime.to_path_buf())
}
