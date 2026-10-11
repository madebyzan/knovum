use std::sync::{Arc, Mutex, OnceLock};

use tokio_util::sync::CancellationToken;

use crate::foundation::app_error::AppError;
use crate::foundation::job_registry::{JobLease, JobRegistry};

#[derive(Default)]
pub(super) struct Generation {
    cancellation: CancellationToken,
    publishing: Mutex<bool>,
}

impl Generation {
    pub(super) fn cancellation(&self) -> &CancellationToken {
        &self.cancellation
    }

    pub(super) fn begin_publication(&self) -> Result<(), AppError> {
        let mut publishing = self
            .publishing
            .lock()
            .map_err(|_| "Image layer publication lock poisoned")?;
        if self.cancellation.is_cancelled() {
            return Err(AppError::cancelled("Image layer split was cancelled"));
        }
        if *publishing {
            return Err("Image layer publication already started".into());
        }
        *publishing = true;
        Ok(())
    }

    fn cancel(&self) -> Result<(), String> {
        let publishing = self
            .publishing
            .lock()
            .map_err(|_| "Image layer publication lock poisoned")?;
        if *publishing {
            return Err("Image layer publication has started and cannot be cancelled".into());
        }
        self.cancellation.cancel();
        Ok(())
    }
}

fn registry() -> &'static JobRegistry<Arc<Generation>> {
    static REGISTRY: OnceLock<JobRegistry<Arc<Generation>>> = OnceLock::new();
    REGISTRY.get_or_init(|| JobRegistry::new("Image layer split"))
}

pub fn acquire(
    generation_id: &str,
) -> Result<(Arc<Generation>, JobLease<Arc<Generation>>), String> {
    let generation = Arc::new(Generation::default());
    let lease = registry().acquire(generation_id, generation.clone())?;
    Ok((generation, lease))
}

pub fn cancel(generation_id: &str) -> Result<(), String> {
    registry().get(generation_id)?.cancel()
}

#[cfg(test)]
#[path = "tests/publication.rs"]
mod tests;
