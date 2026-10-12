use super::*;

#[test]
fn embedding_profile_matches_compiled_backend() {
    let profile = super::super::embedding::profile().unwrap();
    let device = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "mps"
    } else {
        "cpu"
    };
    assert_eq!(profile.device, device);
    assert_eq!(
        profile.execution_profile_id,
        crate::ai::contracts::execution_profile::embedding(device, profile.cpu_threads)
    );
    assert!(memory_requirement(RuntimeCapability::Embedding, &profile).is_ok());
}
