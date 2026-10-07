use crate::ai::contracts::runtime::{RuntimeCapability, RuntimeDeviceProfile};
use crate::local_resources::runtimes::python::RuntimeProfile;

pub(super) fn for_execution(
    capability: RuntimeCapability,
    device: &RuntimeDeviceProfile,
) -> Result<RuntimeProfile, String> {
    use RuntimeCapability as C;
    use RuntimeProfile as P;
    match (capability, device.execution_profile_id.as_str()) {
        (C::Embedding | C::SpeechAlignment | C::SpeechDiarization, _) => Ok(P::TorchInference),
        (C::ImageSegmentation | C::SpeechSeparation, _) => Ok(P::LightInference),
        (C::SpeechTranscription, "mlx-whisper-small-mps-v1") => Ok(P::TorchInference),
        (C::SpeechTranscription, "faster-whisper-small-cpu-int8-v1") => Ok(P::LightInference),
        (C::SpeechSynthesis, "voxcpm2-mps-f32-v1" | "voxcpm2-cpu-bf16-v1") => {
            Ok(P::VoiceGeneration)
        }
        (C::SpeechSynthesis, id) if id.starts_with("cpu-int8-threads") => Ok(P::LightInference),
        _ => Err("runtime_mismatch: execution profile has no runtime package".into()),
    }
}

#[cfg(test)]
#[path = "tests/runtime_profile.rs"]
mod tests;
