use super::*;
use crate::ai::contracts::execution_profile;

mod embedding;

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
            profile("cpu", &execution_profile::embedding("cpu", 4), 4),
            4,
        ),
        (
            RuntimeCapability::Embedding,
            profile("mps", &execution_profile::embedding("mps", 4), 4),
            4,
        ),
        (
            RuntimeCapability::Embedding,
            profile("cpu", &execution_profile::embedding("cpu", 2), 2),
            4,
        ),
        (
            RuntimeCapability::ImageSegmentation,
            profile(
                "cpu",
                &execution_profile::image(RuntimeCapability::ImageSegmentation, 2).unwrap(),
                2,
            ),
            1,
        ),
        (
            RuntimeCapability::SpeechTranscription,
            profile(
                "mps",
                &execution_profile::whisper("small", "mps", "q8_0"),
                4,
            ),
            3,
        ),
        (
            RuntimeCapability::SpeechTranscription,
            profile(
                "cpu",
                &execution_profile::whisper("small", "cpu", "q8_0"),
                4,
            ),
            3,
        ),
        (
            RuntimeCapability::SpeechAlignment,
            profile("cpu", "qwen3-forced-aligner-0.6b-f16-cpu-bundled-v2", 4),
            6,
        ),
        (
            RuntimeCapability::SpeechAlignment,
            profile("mps", "qwen3-forced-aligner-0.6b-f16-mps-bundled-v2", 4),
            6,
        ),
        (
            RuntimeCapability::SpeechAlignment,
            profile(
                "vulkan",
                "qwen3-forced-aligner-0.6b-f16-vulkan-bundled-v2",
                4,
            ),
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
            profile("cpu", &execution_profile::moss_nano(1), 1),
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
            profile("mps", &execution_profile::embedding("cpu", 4), 4),
        ),
        (
            RuntimeCapability::Embedding,
            profile("cpu", &execution_profile::embedding("cpu", 4), 3),
        ),
        (
            RuntimeCapability::Embedding,
            profile("cpu", &execution_profile::embedding("cpu", 5), 5),
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

/// The worker derives the same canonical identities, so an unadmitted one means a dead capability.
#[test]
fn every_canonical_threaded_profile_has_a_memory_manifest() {
    for threads in 1..=4 {
        for device in ["cpu", "mps"] {
            let value = profile(
                device,
                &execution_profile::embedding(device, threads),
                threads,
            );
            assert!(memory_requirement(RuntimeCapability::Embedding, &value).is_ok());
        }
        let value = profile("cpu", &execution_profile::moss_nano(threads), threads);
        assert!(memory_requirement(RuntimeCapability::SpeechSynthesis, &value).is_ok());
        for capability in [
            RuntimeCapability::ImageSegmentation,
            RuntimeCapability::ImageUpscaling,
            RuntimeCapability::ImageInpainting,
            RuntimeCapability::ImageSelection,
            RuntimeCapability::ImageDepth,
        ] {
            let id = execution_profile::image(capability, threads).unwrap();
            assert!(memory_requirement(capability, &profile("cpu", &id, threads)).is_ok());
        }
    }
}

#[test]
fn inference_budget_preserves_system_memory_and_caps_inference_share() {
    assert_eq!(inference_memory_budget(3 * GIB), 0);
    assert_eq!(inference_memory_budget(12 * GIB), 8 * GIB);
    assert_eq!(inference_memory_budget(16 * GIB), 16 * GIB * 2 / 3);
    assert_eq!(inference_memory_budget(48 * GIB), 32 * GIB);
}

#[test]
fn alignment_model_profiles_reject_unqualified_devices_before_execution() {
    use crate::local_resources::model_catalog::LocalModelId as M;
    for model in [M::Qwen3Aligner, M::Qwen3AlignerQ8] {
        for device in ["cpu", "mps", "vulkan"] {
            let id = super::alignment::execution_id(model, device).unwrap();
            assert!(id.ends_with("-bundled-v2"));
            let value = profile(device, &id, 2);
            assert_eq!(
                super::memory_requirement(RuntimeCapability::SpeechAlignment, &value)
                    .unwrap()
                    .peak_bytes(),
                6 * super::GIB
            );
            for retired in ["-native-v2", "-inference-v2"] {
                let obsolete = id.replace("-bundled-v2", retired);
                assert!(super::memory_requirement(
                    RuntimeCapability::SpeechAlignment,
                    &profile(device, &obsolete, 2),
                )
                .is_err());
            }
        }
    }
    assert!(super::alignment::execution_id(M::Qwen3Aligner, "cuda").is_err());
    assert!(super::alignment::execution_id(M::Qwen3AlignerQ8, "invalid").is_err());
    assert!(super::alignment::execution_id(M::MossNano, "cpu").is_err());
}
