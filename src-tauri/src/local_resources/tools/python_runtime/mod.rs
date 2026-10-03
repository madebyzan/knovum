use std::{
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use tauri::{AppHandle, Runtime};

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
}

pub(crate) fn verify_before_spawn(runtime: &PythonRuntimePaths) -> Result<(), String> {
    let bytes = verification::read_manifest(&runtime.manifest_path)?;
    let digest = verification::digest(&bytes);
    if digest != runtime.digest {
        return Err("python-runtime-invalid: runtime manifest changed after resolution".into());
    }
    let current = verification::verify_runtime(&runtime.manifest_path, &bytes, digest)?;
    if current.executable != runtime.executable || current.package_root != runtime.package_root {
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
    let manifest_path = configured_manifest()?.or_else(|| {
        super::bundled_runtime_dir(app).map(|directory| {
            directory
                .join(super::platform_directory())
                .join("python-runtime")
                .join(MANIFEST_NAME)
        })
    });
    let Some(manifest_path) = manifest_path else {
        return Ok(None);
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

    let paths = verification::verify_runtime(&manifest_path, &manifest_bytes, digest)?;
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
