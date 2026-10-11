use crate::foundation::app_error::AppError;
use crate::foundation::job_registry::{JobAdmissionReservation, JobLease, JobRegistry};
use std::sync::OnceLock;
use tokio_util::sync::CancellationToken;

fn jobs() -> &'static JobRegistry {
    static JOBS: OnceLock<JobRegistry> = OnceLock::new();
    JOBS.get_or_init(|| JobRegistry::new("JSON Canvas import"))
}

pub(super) fn reserve_maintenance() -> Result<JobAdmissionReservation, String> {
    jobs().reserve_maintenance()
}

pub(super) fn begin(
    operation_id: &str,
) -> Result<(CancellationToken, JobLease<CancellationToken>), String> {
    jobs().acquire_cancellable(operation_id)
}

pub(super) fn cancel(operation_id: &str) -> Result<(), String> {
    jobs().cancel(operation_id).map(|_| ())
}

pub(super) fn check(token: &CancellationToken) -> Result<(), AppError> {
    if token.is_cancelled() {
        Err(AppError::new(
            "json_canvas_import_cancelled",
            "Canvas import cancelled",
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
