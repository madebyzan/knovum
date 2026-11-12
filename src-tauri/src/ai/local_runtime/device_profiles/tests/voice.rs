use super::*;
use crate::local_resources::model_catalog::{
    VOXCPM2_BF16_SPEC_ID as BF16_MODEL_SPEC_ID, VOXCPM2_FULL_SPEC_ID as FULL_MODEL_SPEC_ID,
    VOXCPM2_Q8_SPEC_ID as Q8_MODEL_SPEC_ID,
};

#[test]
fn usable_metal_is_selected_before_cpu() {
    let memory = super::super::MemorySnapshot {
        total_bytes: 48 * super::super::GIB,
        available_bytes: 30 * super::super::GIB,
    };
    assert_eq!(
        select_profile("mps", FULL_MODEL_SPEC_ID, memory, 4)
            .unwrap()
            .device,
        "mps"
    );
}

#[test]
fn insufficient_gpu_budget_selects_cpu_before_execution() {
    let memory = super::super::MemorySnapshot {
        total_bytes: 32 * super::super::GIB,
        available_bytes: 24 * super::super::GIB,
    };
    let profile = select_profile("mps", FULL_MODEL_SPEC_ID, memory, 4).unwrap();
    assert_eq!(profile.device, "cpu");
    assert_eq!(profile.execution_profile_id, "voxcpm2-cpu-gguf-orig-v1");
}

#[test]
fn bf16_profile_has_its_own_frozen_identity() {
    let memory = super::super::MemorySnapshot {
        total_bytes: 48 * super::super::GIB,
        available_bytes: 30 * super::super::GIB,
    };
    let profile = select_profile("mps", BF16_MODEL_SPEC_ID, memory, 4).unwrap();
    assert_eq!(profile.device, "mps");
    assert_eq!(profile.execution_profile_id, "voxcpm2-mps-gguf-bf16-v1");
}

#[test]
fn q8_profile_has_its_own_frozen_identity_with_the_qualified_conservative_budget() {
    let memory = super::super::MemorySnapshot {
        total_bytes: 48 * super::super::GIB,
        available_bytes: 30 * super::super::GIB,
    };
    let profile = select_profile("mps", Q8_MODEL_SPEC_ID, memory, 4).unwrap();
    assert_eq!(profile.device, "mps");
    assert_eq!(profile.execution_profile_id, "voxcpm2-mps-gguf-q8_0-v1");
}

#[test]
fn cloning_families_are_profiled_by_backend_and_carry_their_footprint() {
    let memory = super::super::MemorySnapshot {
        total_bytes: 48 * super::super::GIB,
        available_bytes: 30 * super::super::GIB,
    };
    let profile = select_profile("vulkan", VoiceVariant::IndexTts2.spec_id(), memory, 4).unwrap();
    assert_eq!(
        profile.execution_profile_id,
        "index-tts2-q8_0-vulkan-bundled-v2"
    );
    assert_eq!(
        cloning_footprint(&profile.execution_profile_id, "vulkan").map(|(_, peak, _)| peak),
        Some(8)
    );
    assert!(select_profile("mps", "unknown-voice-v1", memory, 4).is_err());
}
