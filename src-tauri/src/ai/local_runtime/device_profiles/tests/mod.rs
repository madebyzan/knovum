use super::*;

fn profile(device: &str, id: &str, threads: u8) -> RuntimeDeviceProfile {
    RuntimeDeviceProfile {
        device: device.into(),
        execution_profile_id: id.into(),
        cpu_threads: threads,
    }
}

#[test]
fn exact_profiles_resolve_to_their_declared_peaks() {
    let cases = [
        (
            RuntimeCapability::Embedding,
            profile("cpu", "cpu-f32-threads4-v1", 4),
            4,
        ),
        (
            RuntimeCapability::Embedding,
            profile("mps", "mps-f32-threads4-v1", 4),
            4,
        ),
        (
            RuntimeCapability::Embedding,
            profile("cuda", "cuda-f32-threads2-v1", 2),
            4,
        ),
        (
            RuntimeCapability::ImageSegmentation,
            profile("cpu", "cpu-f32-threads2-v1", 2),
            1,
        ),
        (
            RuntimeCapability::SpeechTranscription,
            profile("mps", "mlx-whisper-small-mps-v1", 4),
            3,
        ),
        (
            RuntimeCapability::SpeechTranscription,
            profile("cpu", "faster-whisper-small-cpu-int8-v1", 4),
            3,
        ),
        (
            RuntimeCapability::SpeechAlignment,
            profile("cpu", "qwen3-forced-aligner-0.6b-hf-cpu-v1", 4),
            6,
        ),
        (
            RuntimeCapability::SpeechAlignment,
            profile("mps", "qwen3-forced-aligner-0.6b-hf-mps-v1", 4),
            6,
        ),
        (
            RuntimeCapability::SpeechAlignment,
            profile("cuda", "qwen3-forced-aligner-0.6b-hf-cuda-v1", 4),
            6,
        ),
        (
            RuntimeCapability::SpeechSeparation,
            profile("cpu", "spleeter-2stems-fp16-cpu-v1", 4),
            8,
        ),
        (
            RuntimeCapability::SpeechSeparation,
            profile("cpu", "spleeter-2stems-fp32-cpu-v1", 4),
            8,
        ),
        (
            RuntimeCapability::SpeechDiarization,
            profile("cpu", "sherpa-diarization-cpu-v1", 4),
            2,
        ),
        (
            RuntimeCapability::SpeechDiarization,
            profile("cpu", "sherpa-diarization-fp32-cpu-v1", 4),
            2,
        ),
        (
            RuntimeCapability::SpeechSynthesis,
            profile("mps", "voxcpm2-mps-gguf-orig-v1", 4),
            24,
        ),
        (
            RuntimeCapability::SpeechSynthesis,
            profile("cpu", "voxcpm2-cpu-gguf-orig-v1", 4),
            16,
        ),
        (
            RuntimeCapability::SpeechSynthesis,
            profile("mps", "voxcpm2-mps-gguf-q8_0-v1", 4),
            24,
        ),
        (
            RuntimeCapability::SpeechSynthesis,
            profile("cpu", "voxcpm2-cpu-gguf-q8_0-v1", 4),
            16,
        ),
        (
            RuntimeCapability::SpeechSynthesis,
            profile("cpu", "moss-nano-cpu-f32-threads1-v1", 1),
            2,
        ),
    ];
    for (capability, profile, peak_gib) in cases {
        assert_eq!(
            memory_requirement(capability, &profile)
                .unwrap()
                .peak_bytes(),
            peak_gib * GIB
        );
    }
}

#[test]
fn profile_manifest_rejects_device_thread_and_capability_mismatches() {
    let cases = [
        (
            RuntimeCapability::Embedding,
            profile("mps", "cpu-f32-threads4-v1", 4),
        ),
        (
            RuntimeCapability::Embedding,
            profile("cpu", "cpu-f32-threads4-v1", 3),
        ),
        (
            RuntimeCapability::Embedding,
            profile("cpu", "cpu-f32-threads5-v1", 5),
        ),
        (
            RuntimeCapability::SpeechAlignment,
            profile("cpu", "sherpa-diarization-cpu-v1", 4),
        ),
        (
            RuntimeCapability::SpeechSynthesis,
            profile("cpu", "unknown", 4),
        ),
    ];
    for (capability, profile) in cases {
        assert_eq!(
            memory_requirement(capability, &profile).unwrap_err(),
            "runtime_mismatch: execution profile has no memory manifest"
        );
    }
}

#[test]
fn application_budget_preserves_system_memory_and_caps_inference_share() {
    assert_eq!(application_memory_budget(3 * GIB), 0);
    assert_eq!(application_memory_budget(12 * GIB), 8 * GIB);
    assert_eq!(application_memory_budget(16 * GIB), 16 * GIB * 2 / 3);
    assert_eq!(application_memory_budget(48 * GIB), 32 * GIB);
}

#[test]
fn alignment_model_profiles_reject_unqualified_devices_before_execution() {
    use crate::local_resources::model_catalog::LocalModelId as M;
    for device in ["cpu", "mps", "cuda", "vulkan"] {
        let id = super::alignment::execution_id(M::Qwen3AlignerQ8, device).unwrap();
        let value = profile(device, &id, 2);
        assert_eq!(
            super::memory_requirement(RuntimeCapability::SpeechAlignment, &value)
                .unwrap()
                .peak_bytes(),
            6 * super::GIB
        );
    }
    assert!(super::alignment::execution_id(M::Qwen3Aligner, "vulkan").is_err());
    assert!(super::alignment::execution_id(M::Qwen3AlignerQ8, "invalid").is_err());
    assert!(super::alignment::execution_id(M::MossNano, "cpu").is_err());
}
