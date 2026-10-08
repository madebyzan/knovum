use crate::{
    ai::contracts::runtime::RuntimeDeviceProfile,
    local_resources::runtimes::python::PythonRuntimePaths,
};
use tokio_util::sync::CancellationToken;

pub(crate) async fn profile(
    runtime: &PythonRuntimePaths,
    cancellation: &CancellationToken,
    exit: &CancellationToken,
    deadline_ms: u64,
) -> Result<RuntimeDeviceProfile, String> {
    let device = super::torch::preferred_device(
        runtime,
        super::EMBEDDING_PEAK_GIB * super::GIB,
        cancellation,
        exit,
        deadline_ms,
    )
    .await?;
    let threads = crate::ai::local_runtime::compute_budget::ComputeBudget::detect()?.model_threads;
    Ok(RuntimeDeviceProfile {
        device: device.into(),
        execution_profile_id: format!("{device}-f32-threads{threads}-v1"),
        cpu_threads: threads,
    })
}
