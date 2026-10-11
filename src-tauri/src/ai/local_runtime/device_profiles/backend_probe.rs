use crate::local_resources::runtimes::native::NativeRuntimePaths;
use serde::de::DeserializeOwned;
use std::{process::Stdio, time::Duration};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy)]
pub(in crate::ai::local_runtime) enum Probe {
    CompiledLibrary(u64),
}
pub(in crate::ai::local_runtime) async fn report<T: DeserializeOwned>(
    runtime: &NativeRuntimePaths,
    probe: Probe,
    cancellation: &CancellationToken,
    exit: &CancellationToken,
    deadline_ms: u64,
) -> Result<T, String> {
    let bytes = detect(runtime, probe, cancellation, exit, deadline_ms).await?;
    serde_json::from_slice(&bytes)
        .map_err(|_| "runtime_mismatch: GPU backend report is invalid".into())
}

async fn detect(
    runtime: &NativeRuntimePaths,
    probe: Probe,
    cancellation: &CancellationToken,
    exit: &CancellationToken,
    deadline_ms: u64,
) -> Result<Vec<u8>, String> {
    tokio::select! {
        biased;
        _ = exit.cancelled() => return Err("cancelled".into()),
        _ = cancellation.cancelled() => return Err("cancelled".into()),
        verified = tokio::time::timeout(super::super::deadline_duration(deadline_ms)?,
            tokio::task::spawn_blocking({let runtime=runtime.clone();move || crate::local_resources::runtimes::native::verify_before_spawn(&runtime)})) =>
            verified.map_err(|_| "deadline_exceeded: GPU probe verification timed out")?.map_err(|_|"runtime_mismatch: native verification task stopped")??,
    }
    let Probe::CompiledLibrary(reserve) = probe;
    let mut command = Command::new(&runtime.executable);
    command
        .args([
            "--audio-devices",
            runtime
                .manifest_path
                .to_str()
                .ok_or("runtime_mismatch: native manifest path is invalid")?,
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
        output = tokio::time::timeout(super::super::deadline_duration(deadline_ms)?.min(Duration::from_secs(15)),
            command.output()) => output.map_err(|_| "deadline_exceeded: GPU detection timed out")?
            .map_err(|_| "runtime_mismatch: GPU detection could not start")?,
    };
    if !output.status.success() || output.stdout.len() > 4096 {
        return Err("runtime_mismatch: GPU backend detection failed".into());
    }
    Ok(output.stdout)
}
