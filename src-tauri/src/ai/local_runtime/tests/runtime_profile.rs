use super::*;

#[test]
fn execution_selects_one_package_before_worker_start() {
    for (capability, id, expected) in [
        (
            RuntimeCapability::Embedding,
            "cpu-f32-threads4-v1",
            RuntimeProfile::TorchInference,
        ),
        (
            RuntimeCapability::SpeechTranscription,
            "mlx-whisper-small-mps-v1",
            RuntimeProfile::TorchInference,
        ),
        (
            RuntimeCapability::SpeechTranscription,
            "faster-whisper-small-cpu-int8-v1",
            RuntimeProfile::LightInference,
        ),
        (
            RuntimeCapability::SpeechSynthesis,
            "cpu-int8-threads4-v1",
            RuntimeProfile::LightInference,
        ),
        (
            RuntimeCapability::SpeechSynthesis,
            "voxcpm2-cpu-bf16-v1",
            RuntimeProfile::VoiceGeneration,
        ),
    ] {
        let device = RuntimeDeviceProfile {
            device: "cpu".into(),
            execution_profile_id: id.into(),
            cpu_threads: 4,
        };
        assert_eq!(for_execution(capability, &device).unwrap(), expected);
    }
    let unknown = RuntimeDeviceProfile {
        device: "cpu".into(),
        execution_profile_id: "unknown".into(),
        cpu_threads: 4,
    };
    assert!(for_execution(RuntimeCapability::SpeechTranscription, &unknown).is_err());
    assert!(serde_json::from_str::<RuntimeProfile>("\"inference\"").is_err());
}
