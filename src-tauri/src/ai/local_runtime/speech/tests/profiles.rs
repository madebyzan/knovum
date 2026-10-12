use super::{transcription_budget_ms, transcription_profile};
use crate::{
    ai::contracts::{
        runtime::{RuntimeCapability, RuntimeDeviceProfile},
        speech_wire::AsrEngine,
    },
    local_resources::model_catalog::LocalModelId,
};
#[test]
fn every_inference_transcription_model_has_a_memory_identity() {
    for model in [
        LocalModelId::WhisperTiny,
        LocalModelId::WhisperSmall,
        LocalModelId::WhisperLargeV3,
    ] {
        let profile = transcription_profile(model).unwrap();
        assert!(matches!(profile.engine, AsrEngine::WhisperCpp));
        assert_eq!(
            profile.compute_type,
            if model == LocalModelId::WhisperLargeV3 {
                "f16"
            } else {
                "q8_0"
            }
        );
        crate::ai::local_runtime::device_profiles::memory_requirement(
            RuntimeCapability::SpeechTranscription,
            &RuntimeDeviceProfile {
                device: profile.device.into(),
                execution_profile_id: profile.id,
                cpu_threads: 1,
            },
        )
        .unwrap();
    }
    assert!(transcription_profile(LocalModelId::MossNano).is_err());
}

#[test]
fn expensive_cpu_asr_is_admitted_without_unbounded_or_non_asr_deadlines() {
    if !cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        assert!(
            transcription_budget_ms(LocalModelId::WhisperLargeV3, 4_000).unwrap()
                > transcription_budget_ms(LocalModelId::WhisperSmall, 4_000).unwrap()
        );
    }
    for model in [LocalModelId::WhisperTiny, LocalModelId::WhisperLargeV3] {
        assert_eq!(
            transcription_budget_ms(model, u64::MAX).unwrap(),
            6 * 60 * 60 * 1_000
        );
    }
    assert!(transcription_budget_ms(LocalModelId::MossNano, 4_000).is_err());
}
