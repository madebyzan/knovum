use crate::ai::contracts::runtime::{RuntimeCapability, RuntimeDeviceProfile};

pub(crate) mod alignment;
pub(in crate::ai::local_runtime) mod backend_probe;
pub(crate) mod embedding;
mod memory;
mod torch;
pub(crate) mod voice;

const GIB: u64 = 1024 * 1024 * 1024;
pub(super) const EMBEDDING_PEAK_GIB: u64 = 4;

#[derive(Debug)]
pub(super) struct MemoryRequirement {
    pub load_peak_bytes: u64,
    pub inference_peak_bytes: u64,
    pub profile_name: &'static str,
}

impl MemoryRequirement {
    pub fn peak_bytes(&self) -> u64 {
        self.load_peak_bytes.max(self.inference_peak_bytes)
    }
}

#[derive(Clone, Copy)]
pub(super) struct MemorySnapshot {
    pub total_bytes: u64,
    pub available_bytes: u64,
}

/// Conservative process peak reservations tied to exact frozen execution profiles.
/// Release qualification must validate them and raise any budget below a measured peak.
pub(in crate::ai::local_runtime) fn memory_requirement(
    capability: RuntimeCapability,
    device_profile: &RuntimeDeviceProfile,
) -> Result<MemoryRequirement, String> {
    let id = device_profile.execution_profile_id.as_str();
    let device = device_profile.device.as_str();
    let (load_peak_gib, inference_peak_gib, profile_name) = match (capability, id, device) {
        (RuntimeCapability::Embedding, _, "cpu" | "mps" | "cuda")
            if valid_threaded_profile(device_profile, &format!("{device}-f32-threads")) =>
        {
            (3, EMBEDDING_PEAK_GIB, "Qwen3 embedding F32")
        }
        (RuntimeCapability::ImageSegmentation, _, "cpu")
            if valid_threaded_profile(device_profile, "cpu-f32-threads") =>
        {
            (1, 1, "U2Net image segmentation CPU")
        }
        (RuntimeCapability::SpeechTranscription, "mlx-whisper-small-mps-v1", "mps") => {
            (2, 3, "MLX Whisper small MPS")
        }
        (RuntimeCapability::SpeechTranscription, "faster-whisper-small-cpu-int8-v1", "cpu")
        | (RuntimeCapability::SpeechTranscription, "faster-whisper-small-cuda-f16-v1", "cuda") => {
            (2, 3, "faster-whisper small CPU INT8")
        }
        (RuntimeCapability::SpeechTranscription, "mlx-whisper-tiny-mps-v1", "mps") => {
            (1, 2, "MLX Whisper tiny MPS")
        }
        (RuntimeCapability::SpeechTranscription, "faster-whisper-tiny-cpu-int8-v1", "cpu")
        | (RuntimeCapability::SpeechTranscription, "faster-whisper-tiny-cuda-f16-v1", "cuda") => {
            (1, 2, "faster-whisper tiny CPU INT8")
        }
        (RuntimeCapability::SpeechTranscription, "mlx-whisper-large-v3-mps-v1", "mps") => {
            (5, 7, "MLX Whisper large-v3 MPS")
        }
        (RuntimeCapability::SpeechTranscription, "faster-whisper-large-v3-cpu-int8-v1", "cpu")
        | (RuntimeCapability::SpeechTranscription, "faster-whisper-large-v3-cuda-f16-v1", "cuda") => {
            (5, 7, "faster-whisper large-v3 CPU INT8")
        }
        (RuntimeCapability::SpeechAlignment, _, "cpu" | "cuda" | "mps")
            if id == format!("qwen3-forced-aligner-0.6b-hf-{device}-v1") =>
        {
            (5, 6, "Qwen3 forced aligner CPU")
        }
        (RuntimeCapability::SpeechAlignment, _, "cpu" | "cuda" | "mps" | "vulkan")
            if id == format!("qwen3-forced-aligner-0.6b-q8_0-{device}-v1") =>
        {
            (5, 6, "Qwen3 forced aligner native Q8")
        }
        (
            RuntimeCapability::SpeechDiarization,
            "sherpa-diarization-cpu-v1" | "sherpa-diarization-fp32-cpu-v1",
            "cpu",
        ) => (1, 2, "Sherpa-ONNX speaker diarization CPU"),
        (
            RuntimeCapability::SpeechSeparation,
            "spleeter-2stems-fp16-cpu-v1" | "spleeter-2stems-fp32-cpu-v1",
            "cpu",
        ) => (2, 8, "Spleeter 2-stem CPU"),
        (RuntimeCapability::SpeechSynthesis, "voxcpm2-mps-gguf-orig-v1", "mps") => {
            (20, 24, "VoxCPM2 native Metal GGUF")
        }
        (RuntimeCapability::SpeechSynthesis, "voxcpm2-cpu-gguf-orig-v1", "cpu")
        | (RuntimeCapability::SpeechSynthesis, "voxcpm2-cuda-gguf-orig-v1", "cuda")
        | (RuntimeCapability::SpeechSynthesis, "voxcpm2-vulkan-gguf-orig-v1", "vulkan") => {
            (14, 16, "VoxCPM2 native GGUF")
        }
        (RuntimeCapability::SpeechSynthesis, "voxcpm2-mps-gguf-q8_0-v1", "mps") => {
            (20, 24, "VoxCPM2 native Q8 Metal GGUF")
        }
        (RuntimeCapability::SpeechSynthesis, "voxcpm2-cpu-gguf-q8_0-v1", "cpu")
        | (RuntimeCapability::SpeechSynthesis, "voxcpm2-cuda-gguf-q8_0-v1", "cuda")
        | (RuntimeCapability::SpeechSynthesis, "voxcpm2-vulkan-gguf-q8_0-v1", "vulkan") => {
            (14, 16, "VoxCPM2 native Q8 GGUF")
        }
        (RuntimeCapability::SpeechSynthesis, _, "cpu")
            if valid_threaded_profile(device_profile, "moss-nano-cpu-f32-threads") =>
        {
            (1, 2, "MOSS-TTS-Nano CPU F32")
        }
        _ => return Err("runtime_mismatch: execution profile has no memory manifest".into()),
    };
    Ok(MemoryRequirement {
        load_peak_bytes: load_peak_gib * GIB,
        inference_peak_bytes: inference_peak_gib * GIB,
        profile_name,
    })
}

fn valid_threaded_profile(profile: &RuntimeDeviceProfile, prefix: &str) -> bool {
    profile
        .execution_profile_id
        .strip_prefix(prefix)
        .and_then(|value| value.strip_suffix("-v1"))
        .and_then(|value| value.parse::<u8>().ok())
        .is_some_and(|threads| (1..=4).contains(&threads) && threads == profile.cpu_threads)
}

pub(super) fn application_memory_budget(total_bytes: u64) -> u64 {
    total_bytes
        .saturating_sub(4 * GIB)
        .min(total_bytes.saturating_mul(2) / 3)
}

pub(super) fn memory_snapshot() -> Result<MemorySnapshot, String> {
    let snapshot = memory::platform_memory_snapshot()?;
    if snapshot.total_bytes == 0 || snapshot.available_bytes == 0 {
        return Err("resource_exhausted: Platform memory statistics are invalid".into());
    }
    Ok(snapshot)
}

pub(crate) fn speech_separation(
    model: crate::local_resources::model_catalog::LocalModelId,
) -> Result<RuntimeDeviceProfile, String> {
    #[cfg(not(knovum_supported_desktop))]
    return Err(
        "unsupported_device: Source separation requires macOS arm64/x64 or Windows x64".into(),
    );
    #[cfg(knovum_supported_desktop)]
    Ok(RuntimeDeviceProfile {
        device: "cpu".into(),
        execution_profile_id: model.separation_spec()?.into(),
        cpu_threads: crate::ai::local_runtime::compute_budget::ComputeBudget::detect()?
            .model_threads,
    })
}

#[cfg(test)]
mod tests;
