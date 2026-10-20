use crate::local_resources::runtimes::inference::InferenceRuntimePaths;
use std::process::Stdio;
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

    /// A cold start reads two disjoint sets of bytes: the host hashes the image it is about to
    /// execute, and the worker hashes the libraries it is about to load and then loads the weights
    /// it was initialized for. Each phase is bounded by the bytes that phase reads, at a throughput
    /// floor far below any supported disk, plus one allowance for the work that is not proportional
    /// to bytes. Neither phase pays for the other's work, and no budget is a constant that a cold
    /// page cache can exhaust.
    fn budget_ms(bytes: u64) -> u64 {
        const VERIFICATION_FLOOR_BYTES_PER_SECOND: u64 = 20 * 1024 * 1024;
        const PROCESS_ALLOWANCE_MS: u64 = 30_000;
        PROCESS_ALLOWANCE_MS.saturating_add(bytes / VERIFICATION_FLOOR_BYTES_PER_SECOND * 1_000)
    }

    /// Bounds [`Self::verify`], which hashes the image the host spawns.
    pub(in crate::ai::local_runtime) fn spawn_verification_budget_ms(&self) -> u64 {
        Self::budget_ms(self.0.spawn_verified_bytes())
    }

    /// Bounds the worker's own initialization: hashing every library before it loads one, and
    /// loading the snapshot it was initialized for. The value travels to the worker in
    /// [`RuntimeInitializeParams::budget_ms`], which makes it the single budget both sides of that
    /// phase honour.
    pub(in crate::ai::local_runtime) fn initialization_budget_ms(&self, model_bytes: u64) -> u64 {
        Self::budget_ms(self.0.library_bytes.saturating_add(model_bytes))
    }

    pub(super) fn digest(&self) -> &str {
        &self.0.digest
    }
    pub(super) fn manifest_path(&self) -> &std::path::Path {
        &self.0.manifest_path
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
