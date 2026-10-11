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
    crate::ai::contracts::speech_wire::alignment_execution_id(model.alignment_spec()?, device)
        .ok_or_else(|| "unsupported_device: forced alignment backend is unsupported".into())
}

pub(crate) async fn profile(
    runtime: &InferenceRuntimePaths,
    model: LocalModelId,
    cancellation: &CancellationToken,
    deadline_ms: u64,
) -> Result<RuntimeDeviceProfile, String> {
    if !matches!(
        model,
        LocalModelId::Qwen3Aligner | LocalModelId::Qwen3AlignerQ8
    ) {
        return Err("Model does not support forced alignment".into());
    }
    let report: Backend = super::backend_probe::report(
        runtime,
        super::backend_probe::Probe::CompiledLibrary(6 * super::GIB),
        cancellation,
        cancellation,
        deadline_ms,
    )
    .await?;
    let device = report.device;
    Ok(RuntimeDeviceProfile {
        execution_profile_id: execution_id(model, &device)?,
        device,
        cpu_threads: super::super::compute_budget::ComputeBudget::detect()?.model_threads,
    })
}
