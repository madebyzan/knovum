use crate::{
    ai::contracts::runtime::{RuntimeCapability, RuntimeDeviceProfile},
    local_resources::runtimes::python::PythonRuntimePaths,
};
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use crate::local_resources::model_catalog::{
    VOXCPM2_FULL_SPEC_ID as FULL_MODEL_SPEC_ID, VOXCPM2_Q8_SPEC_ID as Q8_MODEL_SPEC_ID,
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Backend {
    device: String,
}

pub(crate) fn eligible(model_spec_id: &str) -> Result<bool, String> {
    Ok(super::memory::physical_memory()? >= minimum_gib(model_spec_id)? * super::GIB)
}
pub(crate) fn require_memory(model_spec_id: &str) -> Result<(), String> {
    let minimum_gib = minimum_gib(model_spec_id)?;
    if eligible(model_spec_id)? {
        Ok(())
    } else {
        Err(format!(
            "resource_exhausted: This VoxCPM2 model requires at least {minimum_gib} GiB physical memory"
        ))
    }
}
pub(crate) async fn profile(
    runtime: &PythonRuntimePaths,
    model_spec_id: &str,
    cancellation: &CancellationToken,
) -> Result<RuntimeDeviceProfile, String> {
    require_memory(model_spec_id)?;
    let deadline = crate::clock::unix_time_millis()?
        .checked_add(30_000)
        .ok_or("GPU probe deadline overflow")?;
    let backend: Backend = super::backend_probe::report(
        runtime,
        super::backend_probe::Probe::Native(gpu_reserve_gib(model_spec_id)? * super::GIB),
        cancellation,
        cancellation,
        u64::try_from(deadline).map_err(|_| "GPU probe deadline is invalid")?,
    )
    .await?;
    if !matches!(backend.device.as_str(), "cpu" | "mps" | "cuda" | "vulkan") {
        return Err("runtime_mismatch: Native voice selected an unsupported backend".into());
    }
    select_profile(
        &backend.device,
        model_spec_id,
        super::memory_snapshot()?,
        crate::ai::local_runtime::compute_budget::ComputeBudget::detect()?.model_threads,
    )
}

fn select_profile(
    device: &str,
    model_spec_id: &str,
    memory: super::MemorySnapshot,
    threads: u8,
) -> Result<RuntimeDeviceProfile, String> {
    let quantization = match model_spec_id {
        FULL_MODEL_SPEC_ID => "orig",
        Q8_MODEL_SPEC_ID => "q8_0",
        _ => return Err("runtime_mismatch: VoxCPM2 model specification is unsupported".into()),
    };
    let make = |device: &str| RuntimeDeviceProfile {
        execution_profile_id: format!("voxcpm2-{device}-gguf-{quantization}-v1"),
        device: device.into(),
        cpu_threads: threads,
    };
    let selected = make(device);
    let peak =
        super::memory_requirement(RuntimeCapability::SpeechSynthesis, &selected)?.peak_bytes();
    if device != "cpu"
        && (peak > super::application_memory_budget(memory.total_bytes)
            || peak > memory.available_bytes)
    {
        return Ok(make("cpu"));
    }
    Ok(selected)
}

fn minimum_gib(model_spec_id: &str) -> Result<u64, String> {
    // Q8 keeps the qualified full-model ceiling until target-platform measurements justify
    // a lower reservation. File size alone is not a process peak-memory contract.
    match model_spec_id {
        FULL_MODEL_SPEC_ID | Q8_MODEL_SPEC_ID => Ok(32),
        _ => Err("runtime_mismatch: VoxCPM2 model specification is unsupported".into()),
    }
}

fn gpu_reserve_gib(model_spec_id: &str) -> Result<u64, String> {
    match model_spec_id {
        FULL_MODEL_SPEC_ID | Q8_MODEL_SPEC_ID => Ok(8),
        _ => Err("runtime_mismatch: VoxCPM2 model specification is unsupported".into()),
    }
}

#[cfg(test)]
#[path = "tests/voice.rs"]
mod tests;
