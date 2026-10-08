use crate::local_resources::runtimes::python::PythonRuntimePaths;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Backends {
    cuda: bool,
    mps: bool,
}

impl Backends {
    fn preferred_device(&self) -> &'static str {
        if self.cuda {
            "cuda"
        } else if self.mps {
            "mps"
        } else {
            "cpu"
        }
    }
}

pub(super) async fn preferred_device(
    runtime: &PythonRuntimePaths,
    minimum_memory: u64,
    cancellation: &CancellationToken,
    exit: &CancellationToken,
    deadline_ms: u64,
) -> Result<&'static str, String> {
    let backends: Backends = super::backend_probe::report(
        runtime,
        super::backend_probe::Probe::Torch(minimum_memory),
        cancellation,
        exit,
        deadline_ms,
    )
    .await?;
    Ok(backends.preferred_device())
}

#[cfg(test)]
#[path = "tests/embedding.rs"]
mod tests;
