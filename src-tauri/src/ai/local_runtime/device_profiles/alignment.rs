use crate::{
    ai::contracts::runtime::RuntimeDeviceProfile,
    local_resources::{model_catalog::LocalModelId, runtimes::python::PythonRuntimePaths},
};
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Backend {
    device: String,
}

pub(crate) fn execution_id(model: LocalModelId, device: &str) -> Result<String, String> {
    let specification = model.alignment_spec()?;
    let prefix = specification
        .strip_suffix("-v1")
        .expect("Alignment specifications own a version suffix");
    let valid = match model {
        LocalModelId::Qwen3Aligner => matches!(device, "cpu" | "cuda" | "mps"),
        LocalModelId::Qwen3AlignerQ8 => matches!(device, "cpu" | "cuda" | "mps" | "vulkan"),
        _ => false,
    };
    if !valid {
        return Err("unsupported_device: forced alignment backend is unsupported".into());
    }
    Ok(format!("{prefix}-{device}-v1"))
}

pub(crate) async fn profile(
    runtime: &PythonRuntimePaths,
    model: LocalModelId,
    cancellation: &CancellationToken,
    deadline_ms: u64,
) -> Result<RuntimeDeviceProfile, String> {
    let device = match model {
        LocalModelId::Qwen3Aligner => super::torch::preferred_device(
            runtime,
            6 * super::GIB,
            cancellation,
            cancellation,
            deadline_ms,
        )
        .await?
        .to_owned(),
        LocalModelId::Qwen3AlignerQ8 => {
            let report: Backend = super::backend_probe::report(
                runtime,
                super::backend_probe::Probe::Native(6 * super::GIB),
                cancellation,
                cancellation,
                deadline_ms,
            )
            .await?;
            report.device
        }
        _ => return Err("Model does not support forced alignment".into()),
    };
    Ok(RuntimeDeviceProfile {
        execution_profile_id: execution_id(model, &device)?,
        device,
        cpu_threads: super::super::compute_budget::ComputeBudget::detect()?.model_threads,
    })
}
