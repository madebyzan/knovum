use crate::{
    ai::contracts::runtime::{RuntimeCapability, RuntimeDeviceProfile},
    local_resources::runtimes::inference::InferenceRuntimePaths,
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

/// Resolves the execution profile this machine can actually run. Memory stays out of it: the
/// profile peak is declared in the memory manifest and enforced once, by admission.
pub(crate) async fn profile(
    runtime: &InferenceRuntimePaths,
    model_spec_id: &str,
    cancellation: &CancellationToken,
) -> Result<RuntimeDeviceProfile, String> {
    let backend: Backend = super::backend_probe::report(
        runtime,
        super::backend_probe::Probe::CompiledLibrary(gpu_reserve_gib(model_spec_id)? * super::GIB),
        cancellation,
        cancellation,
    )
    .await?;
    if !matches!(backend.device.as_str(), "cpu" | "mps" | "vulkan") {
        return Err("runtime_mismatch: Compiled voice selected an unsupported backend".into());
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
        && (peak > super::inference_memory_budget(memory.total_bytes)
            || peak > memory.available_bytes)
    {
        return Ok(make("cpu"));
    }
    Ok(selected)
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
