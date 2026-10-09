use std::{path::PathBuf, process::Stdio, sync::OnceLock};

use tauri::{AppHandle, Runtime};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use super::{RuntimeError, RuntimeProfile};
use crate::{
    local_resources::preparation::RESOURCE_PREPARATION_CANCELLED,
    subprocess::process_tree::ProcessTree,
};

/// Debug builds own their runtime producer. Released apps only use frozen published archives.
pub(super) async fn install<R: Runtime>(
    app: &AppHandle<R>,
    profile: RuntimeProfile,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    super::archive::require_active(cancellation)?;
    static PRODUCER: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    let _producer = tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Err(RESOURCE_PREPARATION_CANCELLED.into()),
        guard = PRODUCER.get_or_init(|| tokio::sync::Mutex::new(())).lock() => guard,
    };
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let manifest = profile.development_manifest();
    let identity = super::identity_directory(&manifest)?;
    let installation_lease = loop {
        if identity.exists() {
            let Some(lease) = super::retention::try_acquire(&identity)? else {
                tokio::select! {
                    biased;
                    _ = cancellation.cancelled() => return Err(RESOURCE_PREPARATION_CANCELLED.into()),
                    () = tokio::time::sleep(std::time::Duration::from_millis(10)) => {}
                }
                continue;
            };
            if !identity.exists() {
                drop(lease);
                continue;
            }
            match verify_identity_async(&manifest, profile, cancellation).await {
                Ok(()) => return retain_prepared_runtime(app, profile, manifest, lease).await,
                Err(error @ RuntimeError::InvalidPublication(_)) => {
                    drop(lease);
                    if let Some(installation) = super::retention::try_acquire_exclusive(&identity)?
                    {
                        break installation;
                    }
                    return Err(format!(
                        "Development runtime is active and cannot be replaced: {error}"
                    ));
                }
                Err(error) => return Err(error.into()),
            }
        }
        if let Some(installation) = super::retention::try_acquire_exclusive(&identity)? {
            break installation;
        }
        tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(RESOURCE_PREPARATION_CANCELLED.into()),
            () = tokio::time::sleep(std::time::Duration::from_millis(10)) => {}
        }
    };
    if prepare_identity_async(&identity, &manifest, profile, cancellation).await? {
        let lease = share_installation(installation_lease).await?;
        return retain_prepared_runtime(app, profile, manifest, lease).await;
    }
    let staging = Staging::create(repository.join(".runtime-development/staging"))?;
    let mut command = Command::new("node");
    command
        .arg(repository.join("scripts/runtime/development.mjs"))
        .args(["--profile", profile.name()])
        .arg("--scratch-parent")
        .arg(&staging.0)
        .current_dir(repository)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .kill_on_drop(true);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.as_std_mut().process_group(0);
    }
    #[cfg(windows)]
    let job = crate::subprocess::windows_job::WindowsJob::new()?;
    let mut child = command
        .spawn()
        .map_err(|error| format!("Local resource preparation could not start: {error}"))?;
    #[cfg(windows)]
    if let Err(error) = job.assign(&child) {
        let _ = child.kill().await;
        return Err(error);
    }
    let mut tree = ProcessTree::new(
        &child,
        #[cfg(windows)]
        job,
    )?;
    let status = tokio::select! {
        biased;
        _ = cancellation.cancelled() => {
            tree.terminate();
            let _ = child.kill().await;
            let _ = child.wait().await;
            return Err(RESOURCE_PREPARATION_CANCELLED.into());
        }
        status = child.wait() => status.map_err(|error| error.to_string())?,
    };
    if !status.success() {
        return Err(format!(
            "Local resource preparation failed ({status}); see the development terminal"
        ));
    }
    super::archive::require_active(cancellation)?;
    let lease = share_installation(installation_lease).await?;
    retain_prepared_runtime(app, profile, manifest, lease).await
}

async fn retain_prepared_runtime<R: Runtime>(
    app: &AppHandle<R>,
    profile: RuntimeProfile,
    manifest: PathBuf,
    lease: std::sync::Arc<super::RuntimeLease>,
) -> Result<(), String> {
    let handle = app.clone();
    crate::blocking::run(move || {
        super::resolve_with_lease(&handle, profile, manifest, lease)?;
        super::retire_unused(&handle, profile)?;
        Ok(())
    })
    .await
}

async fn share_installation(
    installation: super::retention::InstallationLease,
) -> Result<std::sync::Arc<super::RuntimeLease>, String> {
    crate::blocking::run(move || installation.into_shared()).await
}

fn prepare_identity(
    identity: &std::path::Path,
    manifest: &std::path::Path,
    profile: RuntimeProfile,
    cancellation: &CancellationToken,
) -> Result<bool, RuntimeError> {
    if !identity.exists() {
        return Ok(false);
    }
    match verify_identity(manifest, profile, cancellation) {
        Ok(()) => Ok(true),
        Err(error @ RuntimeError::InvalidPublication(_)) => {
            log::warn!("Resetting owned development runtime: {error}");
            crate::local_resources::filesystem::remove_directory(identity)?;
            Ok(false)
        }
        Err(error) => Err(error),
    }
}

async fn prepare_identity_async(
    identity: &std::path::Path,
    manifest: &std::path::Path,
    profile: RuntimeProfile,
    cancellation: &CancellationToken,
) -> Result<bool, RuntimeError> {
    let identity = identity.to_path_buf();
    let manifest = manifest.to_path_buf();
    let cancellation = cancellation.clone();
    crate::blocking::run(move || prepare_identity(&identity, &manifest, profile, &cancellation))
        .await
}

fn verify_identity(
    manifest: &std::path::Path,
    profile: RuntimeProfile,
    cancellation: &CancellationToken,
) -> Result<(), RuntimeError> {
    super::verification::read_manifest(manifest)
        .and_then(|bytes| {
            super::verification::verify_runtime(
                manifest,
                &bytes,
                super::verification::digest(&bytes),
                cancellation,
                None,
                profile,
            )
        })
        .map(|_| ())
}

async fn verify_identity_async(
    manifest: &std::path::Path,
    profile: RuntimeProfile,
    cancellation: &CancellationToken,
) -> Result<(), RuntimeError> {
    let manifest = manifest.to_path_buf();
    let cancellation = cancellation.clone();
    crate::blocking::run(move || verify_identity(&manifest, profile, &cancellation)).await
}

// Declare before the process tree: the tree terminates before its owned files are reclaimed.
struct Staging(PathBuf);
impl Staging {
    fn create(root: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let path = root.join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir(&path).map_err(|error| error.to_string())?;
        Ok(Self(path))
    }
}
impl Drop for Staging {
    fn drop(&mut self) {
        if let Err(error) = crate::local_resources::filesystem::remove_directory(&self.0) {
            log::error!("Development runtime staging cleanup failed: {error}");
        }
    }
}

#[cfg(test)]
#[path = "tests/development.rs"]
mod tests;
