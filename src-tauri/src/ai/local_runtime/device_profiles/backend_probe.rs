use crate::ai::local_runtime::process::WorkerPackage;
use crate::local_resources::runtimes::inference::InferenceRuntimePaths;
use serde::de::DeserializeOwned;
use std::{process::Stdio, time::Duration};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

/// The probe process only prints its backend inventory, so it is bounded on its own rather than by
/// the request that needed a device profile.
const PROBE_BUDGET: Duration = Duration::from_secs(15);

#[derive(Clone, Copy)]
pub(in crate::ai::local_runtime) enum Probe {
    CompiledLibrary(u64),
}
pub(in crate::ai::local_runtime) async fn report<T: DeserializeOwned>(
    runtime: &InferenceRuntimePaths,
    probe: Probe,
    cancellation: &CancellationToken,
    exit: &CancellationToken,
) -> Result<T, String> {
    let bytes = detect(runtime, probe, cancellation, exit).await?;
    serde_json::from_slice(&bytes)
        .map_err(|_| "runtime_mismatch: GPU backend report is invalid".into())
}

async fn detect(
    runtime: &InferenceRuntimePaths,
    probe: Probe,
    cancellation: &CancellationToken,
    exit: &CancellationToken,
) -> Result<Vec<u8>, String> {
    // Verification reads the same bytes a worker start-up would, so it shares that budget.
    let verification_budget =
        Duration::from_millis(WorkerPackage::from(runtime).initialization_budget_ms());
    tokio::select! {
        biased;
        _ = exit.cancelled() => return Err("cancelled".into()),
        _ = cancellation.cancelled() => return Err("cancelled".into()),
        verified = tokio::time::timeout(verification_budget,
            tokio::task::spawn_blocking({let runtime=runtime.clone();move || crate::local_resources::runtimes::inference::verify_before_spawn(&runtime)})) =>
            verified.map_err(|_| "deadline_exceeded: GPU probe verification timed out")?.map_err(|_|"runtime_mismatch: inference verification task stopped")??,
    }
    let Probe::CompiledLibrary(reserve) = probe;
    let mut command = Command::new(&runtime.executable);
    command
        .args([
            "--audio-devices",
            runtime
                .manifest_path
                .to_str()
                .ok_or("runtime_mismatch: inference manifest path is invalid")?,
            &runtime.digest,
            &reserve.to_string(),
        ])
        .current_dir(
            runtime
                .manifest_path
                .parent()
                .ok_or("runtime_missing")?
                .join("bin"),
        )
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .env("OMP_NUM_THREADS", "1")
        .env("MKL_NUM_THREADS", "1")
        .kill_on_drop(true);
    super::super::process::isolate_environment(command.as_std_mut());
    let output = tokio::select! {
        biased;
        _ = exit.cancelled() => return Err("cancelled".into()),
        _ = cancellation.cancelled() => return Err("cancelled".into()),
        output = tokio::time::timeout(PROBE_BUDGET,
            command.output()) => output.map_err(|_| "deadline_exceeded: GPU detection timed out")?
            .map_err(|_| "runtime_mismatch: GPU detection could not start")?,
    };
    if !output.status.success() || output.stdout.len() > 4096 {
        return Err("runtime_mismatch: GPU backend detection failed".into());
    }
    Ok(output.stdout)
}
