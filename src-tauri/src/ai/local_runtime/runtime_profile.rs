use crate::ai::contracts::runtime::{RuntimeCapability, RuntimeDeviceProfile};
use crate::local_resources::runtimes::python::RuntimeProfile;

pub(super) fn for_execution(
    capability: RuntimeCapability,
    device: &RuntimeDeviceProfile,
) -> Result<RuntimeProfile, String> {
    use RuntimeCapability as C;
    use RuntimeProfile as P;
    match (capability, device.execution_profile_id.as_str()) {
        (C::Embedding, _) => Ok(P::TorchInference),
        (C::SpeechAlignment, id) if id.starts_with("qwen3-forced-aligner-0.6b-hf-") => {
            Ok(P::TorchInference)
        }
        (C::SpeechAlignment, id) if id.starts_with("qwen3-forced-aligner-0.6b-q8_0-") => {
            Ok(P::VoiceGeneration)
        }
        (
            C::ImageSegmentation
            | C::ImageUpscaling
            | C::ImageInpainting
            | C::ImageSelection
            | C::ImageDepth
            | C::SpeechSeparation
            | C::SpeechDiarization,
            _,
        ) => Ok(P::LightInference),
        (
            C::SpeechTranscription,
            "mlx-whisper-small-mps-v1" | "mlx-whisper-tiny-mps-v1" | "mlx-whisper-large-v3-mps-v1",
        ) => Ok(P::TorchInference),
        (
            C::SpeechTranscription,
            "faster-whisper-small-cpu-int8-v1"
            | "faster-whisper-tiny-cpu-int8-v1"
            | "faster-whisper-large-v3-cpu-int8-v1",
        ) => Ok(if cfg!(target_os = "windows") {
            P::TorchInference
        } else {
            P::LightInference
        }),
        (
            C::SpeechTranscription,
            "faster-whisper-small-cuda-f16-v1"
            | "faster-whisper-tiny-cuda-f16-v1"
            | "faster-whisper-large-v3-cuda-f16-v1",
        ) => Ok(P::TorchInference),
        (
            C::SpeechSynthesis,
            "voxcpm2-mps-gguf-orig-v1"
            | "voxcpm2-cpu-gguf-orig-v1"
            | "voxcpm2-cuda-gguf-orig-v1"
            | "voxcpm2-vulkan-gguf-orig-v1"
            | "voxcpm2-mps-gguf-q8_0-v1"
            | "voxcpm2-cpu-gguf-q8_0-v1"
            | "voxcpm2-cuda-gguf-q8_0-v1"
            | "voxcpm2-vulkan-gguf-q8_0-v1",
        ) => Ok(P::VoiceGeneration),
        (C::SpeechSynthesis, id) if id.starts_with("moss-nano-cpu-f32-threads") => {
            Ok(P::LightInference)
        }
        _ => Err("runtime_mismatch: execution profile has no runtime package".into()),
    }
}

#[cfg(test)]
#[path = "tests/runtime_profile.rs"]
mod tests;
