use crate::{
    ai::contracts::runtime::{RuntimeCapability, RuntimeDeviceProfile},
    local_resources::runtimes::inference::InferenceRuntimePaths,
};
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use crate::{
    ai::contracts::execution_profile,
    local_resources::{
        model_catalog::{VOXCPM2_BF16_SPEC_ID, VOXCPM2_FULL_SPEC_ID, VOXCPM2_Q8_SPEC_ID},
        models::catalog::VoiceVariant,
    },
};

/// What a cloning family asks of a machine: GiB while loading, GiB while generating. audio.cpp keeps
/// the weights and graph arenas in one place on every backend, so one figure holds for each.
const CLONING_FOOTPRINTS: &[(VoiceVariant, u64, u64, &str)] = &[
    (VoiceVariant::IndexTts2, 5, 8, "IndexTTS2 audio.cpp Q8 GGUF"),
    (
        VoiceVariant::IndexTts2F16,
        6,
        10,
        "IndexTTS2 audio.cpp F16 GGUF",
    ),
    (
        VoiceVariant::Qwen3Tts,
        4,
        6,
        "Qwen3-TTS 1.7B audio.cpp Q8 GGUF",
    ),
    (
        VoiceVariant::Qwen3TtsBf16,
        6,
        8,
        "Qwen3-TTS 1.7B audio.cpp BF16 GGUF",
    ),
    (
        VoiceVariant::Qwen3TtsSmall,
        3,
        4,
        "Qwen3-TTS 0.6B audio.cpp Q8 GGUF",
    ),
    (
        VoiceVariant::CosyVoice3,
        4,
        6,
        "Fun-CosyVoice3 audio.cpp Q8 GGUF",
    ),
];

/// The memory manifest entry of a cloning-family execution profile.
pub(super) fn cloning_footprint(
    execution_profile_id: &str,
    device: &str,
) -> Option<(u64, u64, &'static str)> {
    CLONING_FOOTPRINTS
        .iter()
        .find(|(variant, ..)| {
            execution_profile::voice(variant.spec_id(), device).as_deref()
                == Some(execution_profile_id)
        })
        .map(|&(_, load, inference, name)| (load, inference, name))
}
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
    let make = |device: &str| -> Result<RuntimeDeviceProfile, String> {
        Ok(RuntimeDeviceProfile {
            execution_profile_id: execution_profile::voice(model_spec_id, device)
                .ok_or("runtime_mismatch: voice model specification is unsupported")?,
            device: device.into(),
            cpu_threads: threads,
        })
    };
    let selected = make(device)?;
    let peak =
        super::memory_requirement(RuntimeCapability::SpeechSynthesis, &selected)?.peak_bytes();
    if device != "cpu"
        && (peak > super::inference_memory_budget(memory.total_bytes)
            || peak > memory.available_bytes)
    {
        return make("cpu");
    }
    Ok(selected)
}

/// The device memory a backend must have free to be offered for this model.
fn gpu_reserve_gib(model_spec_id: &str) -> Result<u64, String> {
    if matches!(
        model_spec_id,
        VOXCPM2_FULL_SPEC_ID | VOXCPM2_BF16_SPEC_ID | VOXCPM2_Q8_SPEC_ID
    ) {
        return Ok(8);
    }
    CLONING_FOOTPRINTS
        .iter()
        .find(|(variant, ..)| variant.spec_id() == model_spec_id)
        .map(|&(_, _, inference, _)| inference)
        .ok_or_else(|| "runtime_mismatch: voice model specification is unsupported".into())
}

#[cfg(test)]
#[path = "tests/voice.rs"]
mod tests;
