use std::{path::PathBuf, process::Stdio, sync::OnceLock};

use tauri::{AppHandle, Runtime};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use super::RuntimeProfile;
use crate::{
    local_resources::preparation::RESOURCE_PREPARATION_CANCELLED,
    subprocess::process_tree::ProcessTree,
};

/// Debug builds own their native producer. Released apps only use frozen published archives.
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
    let handle = app.clone();
    crate::blocking::run(move || {
        super::resolve(&handle, profile)?.ok_or("Prepared local runtime is missing")?;
        Ok(())
    })
    .await
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
