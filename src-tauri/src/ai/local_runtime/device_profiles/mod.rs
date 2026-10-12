use crate::{
    ai::contracts::{
        execution_profile,
        runtime::{RuntimeCapability, RuntimeDeviceProfile},
    },
    local_resources::model_catalog::LocalModelId,
};

pub(crate) mod alignment;
pub(in crate::ai::local_runtime) mod backend_probe;
pub(crate) mod embedding;
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

pub(super) use crate::local_resources::system_memory::MemorySnapshot;

/// Conservative process peak reservations tied to exact frozen execution profiles.
/// Release qualification must validate them and raise any budget below a measured peak.
pub(in crate::ai::local_runtime) fn memory_requirement(
    capability: RuntimeCapability,
    device_profile: &RuntimeDeviceProfile,
) -> Result<MemoryRequirement, String> {
    let id = device_profile.execution_profile_id.as_str();
    let device = device_profile.device.as_str();
    let (load_peak_gib, inference_peak_gib, profile_name) = match (capability, id, device) {
        (RuntimeCapability::Embedding, _, "cpu" | "mps")
            if threaded_profile(
                device_profile,
                &execution_profile::embedding(device, device_profile.cpu_threads),
            ) =>
        {
            (3, EMBEDDING_PEAK_GIB, "Qwen3 embedding F32")
        }
        (RuntimeCapability::ImageSegmentation, _, "cpu")
            if image_profile(device_profile, capability) =>
        {
            (1, 1, "U2Net image segmentation CPU")
        }
        (RuntimeCapability::ImageUpscaling, _, "cpu")
            if image_profile(device_profile, capability) =>
        {
            (1, 3, "Real-ESRGAN tiled CPU")
        }
        (RuntimeCapability::ImageInpainting, _, "cpu")
            if image_profile(device_profile, capability) =>
        {
            (1, 2, "MI-GAN selected region CPU")
        }
        (RuntimeCapability::ImageSelection, _, "cpu")
            if image_profile(device_profile, capability) =>
        {
            (2, 3, "EfficientSAM CPU")
        }
        (RuntimeCapability::ImageDepth, _, "cpu") if image_profile(device_profile, capability) => {
            (1, 2, "Depth Anything Small CPU")
        }
        (RuntimeCapability::SpeechTranscription, _, "cpu" | "mps")
            if id == execution_profile::whisper("tiny", device, "q8_0") =>
        {
            (1, 2, "Whisper.cpp tiny Q8")
        }
        (RuntimeCapability::SpeechTranscription, _, "cpu" | "mps")
            if id == execution_profile::whisper("small", device, "q8_0") =>
        {
            (2, 3, "Whisper.cpp small Q8")
        }
        (RuntimeCapability::SpeechTranscription, _, "cpu" | "mps")
            if id == execution_profile::whisper("large-v3", device, "f16") =>
        {
            (5, 7, "Whisper.cpp large-v3 F16")
        }
        (RuntimeCapability::SpeechAlignment, _, _)
            if alignment::execution_id(LocalModelId::Qwen3Aligner, device)
                .is_ok_and(|expected| expected == id) =>
        {
            (5, 6, "Qwen3 forced aligner F16 GGUF")
        }
        (RuntimeCapability::SpeechAlignment, _, _)
            if alignment::execution_id(LocalModelId::Qwen3AlignerQ8, device)
                .is_ok_and(|expected| expected == id) =>
        {
            (5, 6, "Qwen3 forced aligner Q8 GGUF")
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
            (20, 24, "VoxCPM2 audio.cpp Metal GGUF")
        }
        (RuntimeCapability::SpeechSynthesis, "voxcpm2-cpu-gguf-orig-v1", "cpu")
        | (RuntimeCapability::SpeechSynthesis, "voxcpm2-vulkan-gguf-orig-v1", "vulkan") => {
            (14, 16, "VoxCPM2 audio.cpp GGUF")
        }
        (RuntimeCapability::SpeechSynthesis, "voxcpm2-mps-gguf-q8_0-v1", "mps") => {
            (20, 24, "VoxCPM2 audio.cpp Q8 Metal GGUF")
        }
        (RuntimeCapability::SpeechSynthesis, "voxcpm2-cpu-gguf-q8_0-v1", "cpu")
        | (RuntimeCapability::SpeechSynthesis, "voxcpm2-vulkan-gguf-q8_0-v1", "vulkan") => {
            (14, 16, "VoxCPM2 audio.cpp Q8 GGUF")
        }
        (RuntimeCapability::SpeechSynthesis, _, "cpu")
            if threaded_profile(
                device_profile,
                &execution_profile::moss_nano(device_profile.cpu_threads),
            ) =>
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

/// Threaded profiles are admitted only for thread counts the compute budget can grant.
fn threaded_profile(profile: &RuntimeDeviceProfile, expected: &str) -> bool {
    (1..=4).contains(&profile.cpu_threads) && profile.execution_profile_id == expected
}

fn image_profile(profile: &RuntimeDeviceProfile, capability: RuntimeCapability) -> bool {
    execution_profile::image(capability, profile.cpu_threads)
        .is_some_and(|expected| threaded_profile(profile, &expected))
}

pub(super) fn inference_memory_budget(total_bytes: u64) -> u64 {
    total_bytes
        .saturating_sub(4 * GIB)
        .min(total_bytes.saturating_mul(2) / 3)
}

pub(super) fn memory_snapshot() -> Result<MemorySnapshot, String> {
    let snapshot = crate::local_resources::system_memory::snapshot()?;
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
