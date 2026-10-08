use super::transcription_profile;
#[cfg(knovum_supported_desktop)]
use crate::ai::contracts::{
    runtime::{RuntimeCapability, RuntimeDeviceProfile},
    speech_wire::AsrEngine,
};

#[test]
#[cfg(not(knovum_supported_desktop))]
fn unqualified_targets_are_rejected() {
    assert!(transcription_profile(
        crate::local_resources::model_catalog::LocalModelId::WhisperSmall
    )
    .is_err());
}

#[test]
#[cfg(knovum_supported_desktop)]
fn transcription_profile_matches_the_runtime_memory_manifest() {
    let profile =
        transcription_profile(crate::local_resources::model_catalog::LocalModelId::WhisperSmall)
            .unwrap();
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    assert!(matches!(profile.engine, AsrEngine::MlxWhisper));
    #[cfg(target_arch = "x86_64")]
    assert!(matches!(profile.engine, AsrEngine::FasterWhisper));
    assert!(!profile.compute_type.is_empty());
    crate::ai::local_runtime::device_profiles::memory_requirement(
        RuntimeCapability::SpeechTranscription,
        &RuntimeDeviceProfile {
            device: profile.device.into(),
            execution_profile_id: profile.id.into(),
            cpu_threads: 1,
        },
    )
    .unwrap();
}

#[test]
fn cuda_transcription_has_frozen_model_and_memory_identity() {
    for model in [
        crate::local_resources::model_catalog::LocalModelId::WhisperTiny,
        crate::local_resources::model_catalog::LocalModelId::WhisperSmall,
        crate::local_resources::model_catalog::LocalModelId::WhisperLargeV3,
    ] {
        let profile = super::faster_cuda_profile(model).unwrap();
        assert_eq!(profile.device, "cuda");
        assert_eq!(profile.compute_type, "float16");
        let device = RuntimeDeviceProfile {
            device: profile.device.into(),
            execution_profile_id: profile.id.into(),
            cpu_threads: 1,
        };
        crate::ai::local_runtime::device_profiles::memory_requirement(
            RuntimeCapability::SpeechTranscription,
            &device,
        )
        .unwrap();
        assert_eq!(
            crate::ai::local_runtime::runtime_profile::for_execution(
                RuntimeCapability::SpeechTranscription,
                &device
            )
            .unwrap(),
            crate::local_resources::runtimes::python::RuntimeProfile::TorchInference
        );
    }
}
