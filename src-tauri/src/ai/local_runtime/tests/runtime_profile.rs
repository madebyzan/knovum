use super::*;

#[test]
fn execution_selects_one_package_before_worker_start() {
    for (capability, id, expected) in [
        (
            RuntimeCapability::SpeechAlignment,
            "qwen3-forced-aligner-0.6b-q8_0-cpu-v1",
            RuntimeProfile::VoiceGeneration,
        ),
        (
            RuntimeCapability::SpeechAlignment,
            "qwen3-forced-aligner-0.6b-hf-cpu-v1",
            RuntimeProfile::TorchInference,
        ),
        (
            RuntimeCapability::SpeechDiarization,
            "sherpa-diarization-cpu-v1",
            RuntimeProfile::LightInference,
        ),
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
            if cfg!(windows) {
                RuntimeProfile::TorchInference
            } else {
                RuntimeProfile::LightInference
            },
        ),
        (
            RuntimeCapability::SpeechTranscription,
            "faster-whisper-small-cuda-f16-v1",
            RuntimeProfile::TorchInference,
        ),
        (
            RuntimeCapability::SpeechSynthesis,
            "moss-nano-cpu-f32-threads4-v1",
            RuntimeProfile::LightInference,
        ),
        (
            RuntimeCapability::SpeechSynthesis,
            "voxcpm2-cpu-gguf-orig-v1",
            RuntimeProfile::VoiceGeneration,
        ),
        (
            RuntimeCapability::SpeechSynthesis,
            "voxcpm2-cpu-gguf-q8_0-v1",
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
