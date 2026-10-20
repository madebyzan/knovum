use crate::ai::local_runtime::process::WorkerPackage;
use crate::local_resources::runtimes::inference::InferenceRuntimePaths;
use serde::de::DeserializeOwned;
use std::{process::Stdio, time::Duration};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

/// The probe process hashes the inventory and loads one library, exactly as a worker's
/// initialization does, so it is bounded by that phase rather than by a constant a cold page
/// cache can exhaust. It loads no model and prints only its backend inventory.
const PROBE_MODEL_BYTES: u64 = 0;

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
    // The host hashes the image it is about to execute; the probe process hashes the libraries it
    // loads. Each phase is bounded by the bytes that phase reads.
    let package = WorkerPackage::from(runtime);
    let verification_budget = Duration::from_millis(package.spawn_verification_budget_ms());
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
    let probe_budget = Duration::from_millis(package.initialization_budget_ms(PROBE_MODEL_BYTES));
    let output = tokio::select! {
        biased;
        _ = exit.cancelled() => return Err("cancelled".into()),
        _ = cancellation.cancelled() => return Err("cancelled".into()),
        output = tokio::time::timeout(probe_budget,
            command.output()) => output.map_err(|_| "deadline_exceeded: GPU detection timed out")?
            .map_err(|_| "runtime_mismatch: GPU detection could not start")?,
    };
    if !output.status.success() || output.stdout.len() > 4096 {
        return Err("runtime_mismatch: GPU backend detection failed".into());
    }
    Ok(output.stdout)
}
