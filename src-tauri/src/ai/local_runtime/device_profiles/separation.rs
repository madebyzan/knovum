use crate::{
    ai::contracts::runtime::RuntimeDeviceProfile,
    local_resources::{model_catalog::LocalModelId, runtimes::inference::InferenceRuntimePaths},
};
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Backend {
    device: String,
}

pub(crate) fn execution_id(model: LocalModelId, device: &str) -> Result<String, String> {
    crate::ai::contracts::execution_profile::separation(model.separation_spec()?, device)
        .ok_or_else(|| "unsupported_device: source separation backend is unsupported".into())
}

/// HTDemucs on the GPU backend the bundled audio.cpp library reports, or the CPU.
pub(crate) async fn profile(
    runtime: &InferenceRuntimePaths,
    model: LocalModelId,
    cancellation: &CancellationToken,
) -> Result<RuntimeDeviceProfile, String> {
    model.separation_spec()?;
    let report: Backend = super::backend_probe::report(
        runtime,
        super::backend_probe::Probe::CompiledLibrary(super::SEPARATION_PEAK_GIB * super::GIB),
        cancellation,
        cancellation,
    )
    .await?;
    let device = report.device;
    Ok(RuntimeDeviceProfile {
        execution_profile_id: execution_id(model, &device)?,
        device,
        cpu_threads: super::super::compute_budget::ComputeBudget::detect()?.model_threads,
    })
}
