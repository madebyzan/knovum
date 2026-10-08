use super::*;

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
fn q8_profile_has_its_own_frozen_identity_with_the_qualified_conservative_budget() {
    let memory = super::super::MemorySnapshot {
        total_bytes: 48 * super::super::GIB,
        available_bytes: 30 * super::super::GIB,
    };
    let profile = select_profile("mps", Q8_MODEL_SPEC_ID, memory, 4).unwrap();
    assert_eq!(profile.device, "mps");
    assert_eq!(profile.execution_profile_id, "voxcpm2-mps-gguf-q8_0-v1");
}
