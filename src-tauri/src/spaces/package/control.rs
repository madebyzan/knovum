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
    operation_id: &str,
) -> Result<(CancellationToken, JobLease<CancellationToken>), AppError> {
    validate_operation_id(operation_id)?;
    jobs()
        .acquire_cancellable(operation_id)
        .map_err(AppError::from)
}

pub(super) fn cancel(operation_id: &str) -> Result<(), AppError> {
    validate_operation_id(operation_id)?;
    jobs()
        .cancel(operation_id)
        .map(|_| ())
        .map_err(AppError::from)
}

fn validate_operation_id(operation_id: &str) -> Result<(), AppError> {
    if operation_id.len() != 36 {
        return Err(AppError::invalid(
            "Space package operation ID must be a 36-character UUID",
        ));
    }
    let parsed = uuid::Uuid::parse_str(operation_id)
        .map_err(|_| AppError::invalid("Space package operation ID must be a UUID"))?;
    if parsed.to_string() != operation_id {
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
pub(crate) mod tests;
