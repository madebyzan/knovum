use crate::local_resources::runtimes::python::PythonRuntimePaths;
use serde::de::DeserializeOwned;
use std::{process::Stdio, time::Duration};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy)]
pub(in crate::ai::local_runtime) enum Probe {
    Torch(u64),
    #[cfg(windows)]
    Transcription(u64),
    Native(u64),
}
pub(in crate::ai::local_runtime) async fn report<T: DeserializeOwned>(
    runtime: &PythonRuntimePaths,
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
    runtime: &PythonRuntimePaths,
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
            super::super::process::verify_runtime_for_spawn(runtime.clone())) =>
            verified.map_err(|_| "deadline_exceeded: GPU probe verification timed out")??,
    }
    let module = match probe {
        Probe::Torch(_) => "knovum_inference.runtime.device_probe",
        #[cfg(windows)]
        Probe::Transcription(_) => "knovum_inference.runtime.device_probe",
        Probe::Native(_) => "knovum_inference.runtime.voice_device_probe",
    };
    let script = crate::subprocess::python::isolated_bootstrap(&format!(
        "from {module} import main; main()"
    ));
    let mut command = Command::new(&runtime.executable);
    command
        .args(["-I", "-B", "-X", "utf8", "-c", &script])
        .arg(&runtime.worker_root)
        .arg(&runtime.package_root)
        .arg(
            match probe {
                Probe::Torch(memory) => memory,
                #[cfg(windows)]
                Probe::Transcription(memory) => memory,
                Probe::Native(memory) => memory,
            }
            .to_string(),
        )
        .arg(match probe {
            #[cfg(windows)]
            Probe::Transcription(_) => "transcription",
            _ => "torch",
        })
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .env_remove("PYTHONPATH")
        .env_remove("PYTHONHOME")
        .env("OMP_NUM_THREADS", "1")
        .env("MKL_NUM_THREADS", "1")
        .kill_on_drop(true);
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
