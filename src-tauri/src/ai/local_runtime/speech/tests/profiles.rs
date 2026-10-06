use super::transcription_profile;
#[cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "windows", target_arch = "x86_64")
))]
use crate::ai::contracts::{
    runtime::{RuntimeCapability, RuntimeDeviceProfile},
    speech_wire::AsrEngine,
};

#[test]
#[cfg(not(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "windows", target_arch = "x86_64")
)))]
fn unqualified_targets_are_rejected() {
    assert!(transcription_profile().is_err());
}

#[test]
#[cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "windows", target_arch = "x86_64")
))]
fn transcription_profile_matches_the_runtime_memory_manifest() {
    let profile = transcription_profile().unwrap();
    #[cfg(target_os = "macos")]
    assert!(matches!(profile.engine, AsrEngine::MlxWhisper));
    #[cfg(target_os = "windows")]
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
