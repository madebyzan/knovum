use super::super::{admission::Priority, lease::WorkerLease, process::WorkerSpecification};
use crate::ai::contracts::{
    execution_profile,
    runtime::{RuntimeCapability, RuntimeDeviceProfile, RuntimeModel},
};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

/// Holds the inference slot on the worker that owns one image model.
///
/// Image work is always interactive and always runs on the CPU profile this host detected, which
/// is the only thing the image capabilities decide for themselves; everything else about reaching
/// a worker belongs to [`WorkerLease`].
pub(super) async fn acquire<R: Runtime>(
    app: &AppHandle<R>,
    capability: RuntimeCapability,
    capability_spec_id: &str,
    model: RuntimeModel,
    cancellation: &CancellationToken,
) -> Result<WorkerLease, String> {
    let cpu_threads = super::super::compute_budget::ComputeBudget::detect()?.model_threads;
    WorkerLease::acquire(
        app,
        Priority::Interactive,
        WorkerSpecification {
            capability,
            capability_spec_id: capability_spec_id.into(),
            device_profile: RuntimeDeviceProfile {
                device: "cpu".into(),
                execution_profile_id: execution_profile::image(capability, cpu_threads)
                    .ok_or("runtime_mismatch: not an image capability")?,
                cpu_threads,
            },
            model,
        },
        cancellation,
    )
    .await
}
