use crate::ai::contracts::runtime::RuntimeDeviceProfile;

pub(crate) fn profile() -> Result<RuntimeDeviceProfile, String> {
    let threads = super::super::compute_budget::ComputeBudget::detect()?.model_threads;
    let device = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "mps"
    } else {
        "cpu"
    };
    Ok(RuntimeDeviceProfile {
        device: device.into(),
        execution_profile_id: format!("{device}-f32-threads{threads}-inference-v2"),
        cpu_threads: threads,
    })
}
