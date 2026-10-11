use crate::local_resources::runtimes::{native::NativeRuntimePaths, RuntimeLease};
use std::{process::Stdio, sync::Arc};
use tokio::process::Command;

#[derive(Clone)]
pub(in crate::ai::local_runtime) struct WorkerPackage(NativeRuntimePaths);
impl From<&NativeRuntimePaths> for WorkerPackage {
    fn from(paths: &NativeRuntimePaths) -> Self {
        Self(paths.clone())
    }
}

impl WorkerPackage {
    pub(super) async fn verify(&self) -> Result<(), String> {
        let paths = self.0.clone();
        tokio::task::spawn_blocking(move || {
            crate::local_resources::runtimes::native::verify_before_spawn(&paths)
        })
        .await
        .map_err(|error| format!("Native verification task failed: {error}"))?
    }
    pub(super) fn digest(&self) -> &str {
        &self.0.digest
    }
    pub(super) fn lease(&self) -> Option<Arc<RuntimeLease>> {
        self.0.lease.clone()
    }
    pub(super) fn command(&self, threads: u8) -> Command {
        let mut command = Command::new(&self.0.executable);
        let bin = self
            .0
            .manifest_path
            .parent()
            .expect("Verified native manifest has a parent")
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
