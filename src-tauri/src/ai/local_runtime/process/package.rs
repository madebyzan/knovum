use crate::local_resources::runtimes::inference::InferenceRuntimePaths;
use std::{process::Stdio, time::Duration};
use tokio::process::Command;

#[derive(Clone)]
pub(in crate::ai::local_runtime) struct WorkerPackage(InferenceRuntimePaths);
impl From<&InferenceRuntimePaths> for WorkerPackage {
    fn from(paths: &InferenceRuntimePaths) -> Self {
        Self(paths.clone())
    }
}

impl WorkerPackage {
    pub(super) async fn verify(&self) -> Result<(), String> {
        let paths = self.0.clone();
        tokio::task::spawn_blocking(move || {
            crate::local_resources::runtimes::inference::verify_before_spawn(&paths)
        })
        .await
        .map_err(|error| format!("Inference verification task failed: {error}"))?
    }

    /// A worker hashes the whole runtime before it loads any library, so its initialization budget
    /// is that work plus a fixed allowance for spawning the process and building its engine. The
    /// throughput floor is far below any supported disk, so the budget never races the real cost.
    pub(super) fn initialization_budget(&self) -> Duration {
        const VERIFICATION_FLOOR_BYTES_PER_SECOND: u64 = 20 * 1024 * 1024;
        Duration::from_secs(15)
            + Duration::from_secs(self.0.verified_bytes / VERIFICATION_FLOOR_BYTES_PER_SECOND)
    }

    pub(super) fn digest(&self) -> &str {
        &self.0.digest
    }
    pub(super) fn command(&self, threads: u8) -> Command {
        let mut command = Command::new(&self.0.executable);
        let bin = self
            .0
            .manifest_path
            .parent()
            .expect("Verified inference manifest has a parent")
            .join("bin");
        command
            .current_dir(bin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        super::isolate_environment(command.as_std_mut());
        for name in [
            "OMP_NUM_THREADS",
            "MKL_NUM_THREADS",
            "OPENBLAS_NUM_THREADS",
            "VECLIB_MAXIMUM_THREADS",
            "RAYON_NUM_THREADS",
        ] {
            command.env(name, threads.to_string());
        }
        command
    }
}
