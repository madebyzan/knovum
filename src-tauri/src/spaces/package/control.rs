use std::sync::OnceLock;

use tokio_util::sync::CancellationToken;

use crate::{
    app_error::AppError,
    job_registry::{JobAdmissionReservation, JobLease, JobRegistry},
};

const CANCELLED: &str = "Space package transfer cancelled";

fn jobs() -> &'static JobRegistry {
    static JOBS: OnceLock<JobRegistry> = OnceLock::new();
    JOBS.get_or_init(|| JobRegistry::new("Space package transfer"))
}

pub(crate) fn reserve_maintenance() -> Result<JobAdmissionReservation, String> {
    jobs().reserve_maintenance()
}

pub(super) fn begin(
    id: &str,
) -> Result<(CancellationToken, JobLease<CancellationToken>), AppError> {
    validate_operation_id(id)?;
    jobs().acquire_cancellable(id).map_err(AppError::from)
}

#[cfg(test)]
pub(crate) fn begin_test_job(id: &str) -> Result<JobLease<CancellationToken>, String> {
    begin(id)
        .map(|(_, lease)| lease)
        .map_err(|error| error.message)
}

pub(super) fn cancel(id: &str) -> Result<(), AppError> {
    validate_operation_id(id)?;
    jobs().cancel(id).map_err(AppError::from)
}

fn validate_operation_id(id: &str) -> Result<(), AppError> {
    if id.len() != 36 {
        return Err(AppError::invalid(
            "Space package operation ID must be a 36-character UUID",
        ));
    }
    let parsed = uuid::Uuid::parse_str(id)
        .map_err(|_| AppError::invalid("Space package operation ID must be a UUID"))?;
    if parsed.to_string() != id {
        return Err(AppError::invalid(
            "Space package operation ID must use canonical lowercase UUID form",
        ));
    }
    Ok(())
}

pub(super) fn check(token: &CancellationToken) -> Result<(), AppError> {
    if token.is_cancelled() {
        Err(AppError::cancelled(CANCELLED))
    } else {
        Ok(())
    }
}

pub(super) fn with_cleanup_error(
    mut primary: AppError,
    context: &str,
    cleanup: impl std::fmt::Display,
) -> AppError {
    primary.message = format!("{}; {context}: {cleanup}", primary.message);
    primary
}

#[cfg(test)]
#[path = "tests/control.rs"]
mod tests;
