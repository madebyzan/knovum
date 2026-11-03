use super::super::{admission::Priority, lease::WorkerLease, process::WorkerSpecification};
use crate::ai::contracts::{
    execution_profile,
    runtime::{RuntimeCapability, RuntimeDeviceProfile, RuntimeModel},
};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

/// Holds the inference slot on the worker that owns one image model.
///
/// Image work always runs on the CPU profile this host detected; its priority and that profile are
/// the only things the image capabilities decide for themselves. Everything else about reaching a
/// worker belongs to [`WorkerLease`].
pub(super) async fn acquire<R: Runtime>(
    app: &AppHandle<R>,
    priority: Priority,
    capability: RuntimeCapability,
    capability_spec_id: &str,
    model: RuntimeModel,
    cancellation: &CancellationToken,
) -> Result<WorkerLease, String> {
    let cpu_threads = super::super::compute_budget::ComputeBudget::detect()?.model_threads;
    WorkerLease::acquire(
        app,
        priority,
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
