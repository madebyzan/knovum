use crate::ai::contracts::{execution_profile, runtime::RuntimeDeviceProfile};

pub(crate) fn profile() -> Result<RuntimeDeviceProfile, String> {
    let threads = super::super::compute_budget::ComputeBudget::detect()?.model_threads;
    let device = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "mps"
    } else {
        "cpu"
    };
    Ok(RuntimeDeviceProfile {
        device: device.into(),
        execution_profile_id: execution_profile::embedding(device, threads),
        cpu_threads: threads,
    })
}
