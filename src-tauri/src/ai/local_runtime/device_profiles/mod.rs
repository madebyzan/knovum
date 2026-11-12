use crate::{
    ai::contracts::{
        execution_profile,
        runtime::{RuntimeCapability, RuntimeDeviceProfile},
    },
    local_resources::{model_catalog::LocalModelId, models::catalog::SegmentationModel},
};

pub(crate) mod alignment;
pub(in crate::ai::local_runtime) mod backend_probe;
pub(crate) mod embedding;
pub(crate) mod qwen3_asr;
pub(crate) mod separation;
pub(crate) mod voice;

const GIB: u64 = 1024 * 1024 * 1024;
pub(super) const EMBEDDING_PEAK_GIB: u64 = 4;
/// HTDemucs reads one minute of stereo per call (plus overlap), split by audio.cpp into its own
/// short segments, so its working set does not grow with the source.
const SEPARATION_PEAK_GIB: u64 = 3;
/// Qwen3-ASR is a 1.7B audio-language model: its encoder windows a region while the text decoder
/// generates, so the working set is the weights plus one region's activations, not the source.
const QWEN3_ASR_Q8_PEAK_GIB: u64 = 8;
const QWEN3_ASR_F16_PEAK_GIB: u64 = 12;

fn qwen3_asr_peak_gib(model: LocalModelId) -> Result<u64, String> {
    match model {
        LocalModelId::Qwen3Asr => Ok(QWEN3_ASR_Q8_PEAK_GIB),
        LocalModelId::Qwen3AsrF16 => Ok(QWEN3_ASR_F16_PEAK_GIB),
        _ => Err("Model does not support Qwen3-ASR transcription".into()),
    }
}

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

/// The reservation for one worker: the execution profile it runs under and the model it loads.
/// Background-removal models share one execution profile yet differ several-fold in memory, so
/// their inference peak is the model's own; every other capability's models fit its profile's.
pub(in crate::ai::local_runtime) fn worker_memory_requirement(
    capability: RuntimeCapability,
    model_spec_id: &str,
    device_profile: &RuntimeDeviceProfile,
) -> Result<MemoryRequirement, String> {
    let requirement = memory_requirement(capability, device_profile)?;
    if capability != RuntimeCapability::ImageSegmentation {
        return Ok(requirement);
    }
    let model = SegmentationModel::from_spec_id(model_spec_id)?.spec();
    Ok(MemoryRequirement {
        inference_peak_bytes: model.peak_memory_gib * GIB,
        profile_name: model.snapshot_name,
        ..requirement
    })
}

/// Conservative process peak reservations tied to exact frozen execution profiles.
/// Release qualification must validate them and raise any budget below a measured peak.
pub(in crate::ai::local_runtime) fn memory_requirement(
    capability: RuntimeCapability,
    device_profile: &RuntimeDeviceProfile,
) -> Result<MemoryRequirement, String> {
    let id = device_profile.execution_profile_id.as_str();
    let device = device_profile.device.as_str();
    if let (
        RuntimeCapability::SpeechSynthesis,
        Some((load_peak_gib, inference_peak_gib, profile_name)),
    ) = (capability, voice::cloning_footprint(id, device))
    {
        return Ok(MemoryRequirement {
            load_peak_bytes: load_peak_gib * GIB,
            inference_peak_bytes: inference_peak_gib * GIB,
            profile_name,
        });
    }
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
            // The inference peak is the model's (`worker_memory_requirement`).
            (1, 1, "Image segmentation CPU")
        }
        (RuntimeCapability::ImageUpscaling, _, "cpu")
            if image_profile(device_profile, capability) =>
        {
            (1, 3, "Real-ESRGAN tiled CPU")
        }
        (RuntimeCapability::ImageInpainting, _, "cpu")
            if image_profile(device_profile, capability) =>
        {
            // Big LaMa (fp32, 512² window; measured 1.0 GiB process peak).
            (1, 2, "Image inpainting CPU")
        }
        (RuntimeCapability::ImageSelection, _, "cpu")
            if image_profile(device_profile, capability) =>
        {
            (2, 3, "SAM selection CPU")
        }
        (RuntimeCapability::ImageDepth, _, "cpu") if image_profile(device_profile, capability) => {
            (1, 2, "Depth Anything Small CPU")
        }
        (RuntimeCapability::ImageTextRecognition, _, "cpu")
            if image_profile(device_profile, capability) =>
        {
            (1, 2, "PP-OCRv6 small CPU")
        }
        (RuntimeCapability::SpeechTranscription, _, "cpu" | "mps")
            if id == execution_profile::whisper("tiny", device, "q8_0") =>
        {
            (1, 2, "Whisper.cpp tiny Q8")
        }
        (RuntimeCapability::SpeechTranscription, _, "cpu" | "mps")
            if id == execution_profile::whisper("base", device, "q8_0") =>
        {
            (1, 2, "Whisper.cpp base Q8")
        }
        (RuntimeCapability::SpeechTranscription, _, "cpu" | "mps")
            if id == execution_profile::whisper("small", device, "q8_0") =>
        {
            (2, 3, "Whisper.cpp small Q8")
        }
        (RuntimeCapability::SpeechTranscription, _, "cpu" | "mps")
            if id == execution_profile::whisper("medium", device, "q8_0") =>
        {
            (3, 4, "Whisper.cpp medium Q8")
        }
        (RuntimeCapability::SpeechTranscription, _, "cpu" | "mps")
            if id == execution_profile::whisper("medium", device, "f16") =>
        {
            (4, 5, "Whisper.cpp medium F16")
        }
        (RuntimeCapability::SpeechTranscription, _, "cpu" | "mps")
            if id == execution_profile::whisper("large-v3-turbo", device, "q8_0") =>
        {
            (3, 4, "Whisper.cpp large-v3-turbo Q8")
        }
        (RuntimeCapability::SpeechTranscription, _, "cpu" | "mps")
            if id == execution_profile::whisper("large-v3-turbo", device, "f16") =>
        {
            (4, 5, "Whisper.cpp large-v3-turbo F16")
        }
        (RuntimeCapability::SpeechTranscription, _, "cpu" | "mps")
            if id == execution_profile::whisper("large-v3", device, "f16") =>
        {
            (5, 7, "Whisper.cpp large-v3 F16")
        }
        (RuntimeCapability::SpeechTranscription, _, _)
            if qwen3_asr::execution_id(LocalModelId::Qwen3Asr, device)
                .is_ok_and(|expected| expected == id) =>
        {
            (5, QWEN3_ASR_Q8_PEAK_GIB, "Qwen3-ASR 1.7B Q8 GGUF")
        }
        (RuntimeCapability::SpeechTranscription, _, _)
            if qwen3_asr::execution_id(LocalModelId::Qwen3AsrF16, device)
                .is_ok_and(|expected| expected == id) =>
        {
            (7, QWEN3_ASR_F16_PEAK_GIB, "Qwen3-ASR 1.7B F16 GGUF")
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
            "sherpa-diarization-cpu-v1"
            | "sherpa-diarization-campplus-cpu-v1"
            | "sherpa-diarization-eres2net-large-cpu-v1"
            | "sherpa-diarization-fp32-cpu-v1",
            "cpu",
        ) => (1, 2, "Sherpa-ONNX speaker diarization CPU"),
        (RuntimeCapability::SpeechSeparation, _, _)
            if separation::execution_id(LocalModelId::HtDemucs, device)
                .is_ok_and(|expected| expected == id) =>
        {
            (1, SEPARATION_PEAK_GIB, "HTDemucs Q8 GGUF")
        }
        (RuntimeCapability::SpeechSeparation, _, _)
            if separation::execution_id(LocalModelId::HtDemucsF16, device)
                .is_ok_and(|expected| expected == id) =>
        {
            (1, SEPARATION_PEAK_GIB, "HTDemucs F16 GGUF")
        }
        (RuntimeCapability::SpeechSynthesis, "voxcpm2-mps-gguf-orig-v1", "mps") => {
            (20, 24, "VoxCPM2 audio.cpp Metal GGUF")
        }
        (RuntimeCapability::SpeechSynthesis, "voxcpm2-cpu-gguf-orig-v1", "cpu")
        | (RuntimeCapability::SpeechSynthesis, "voxcpm2-vulkan-gguf-orig-v1", "vulkan") => {
            (14, 16, "VoxCPM2 audio.cpp GGUF")
        }
        (RuntimeCapability::SpeechSynthesis, "voxcpm2-mps-gguf-bf16-v1", "mps") => {
            (20, 24, "VoxCPM2 audio.cpp BF16 Metal GGUF")
        }
        (RuntimeCapability::SpeechSynthesis, "voxcpm2-cpu-gguf-bf16-v1", "cpu")
        | (RuntimeCapability::SpeechSynthesis, "voxcpm2-vulkan-gguf-bf16-v1", "vulkan") => {
            (14, 16, "VoxCPM2 audio.cpp BF16 GGUF")
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

#[cfg(test)]
mod tests;
