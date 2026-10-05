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
            profile("cpu", "qwen3-forced-aligner-0.6b-cpu-v1", 4),
            6,
        ),
        (
            RuntimeCapability::SpeechDiarization,
            profile("cpu", "community1-knovum-cpu-v1", 4),
            4,
        ),
        (
            RuntimeCapability::SpeechSynthesis,
            profile("mps", "voxcpm2-mps-f32-v1", 4),
            24,
        ),
        (
            RuntimeCapability::SpeechSynthesis,
            profile("cpu", "voxcpm2-cpu-bf16-v1", 4),
            16,
        ),
        (
            RuntimeCapability::SpeechSynthesis,
            profile("cpu", "cpu-int8-threads1-v1", 1),
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
            profile("cpu", "community1-knovum-cpu-v1", 4),
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
