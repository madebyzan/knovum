use crate::app_error::AppError;
use std::{future::Future, pin::Pin, sync::Arc, time::Duration};

use tauri::AppHandle;

use super::{scheduler, StepOutcome};

const ERROR_RETRY_MILLIS: i64 = 5_000;
const MAX_CONCURRENT_STEPS: usize = 4;

pub(crate) type ExecuteStep =
    for<'a> fn(
        &'a AppHandle,
        &'a sqlx::SqlitePool,
        &'a super::AttemptClaim,
    ) -> Pin<Box<dyn Future<Output = Result<StepOutcome, AppError>> + Send + 'a>>;
pub(crate) type AfterAttempt =
    for<'a> fn(
        &'a AppHandle,
        &'a sqlx::SqlitePool,
        &'a super::AttemptClaim,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;
pub(crate) type AfterControls =
    for<'a> fn(
        &'a AppHandle,
        &'a sqlx::SqlitePool,
        bool,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;

#[derive(Clone, Copy)]
pub(crate) struct SchedulerHooks {
    pub(crate) execute_step: ExecuteStep,
    pub(crate) after_attempt: AfterAttempt,
    pub(crate) after_controls: AfterControls,
}

pub(crate) fn start(app: AppHandle, hooks: SchedulerHooks) {
    tauri::async_runtime::spawn(async move {
        let executor_id = format!("desktop:{}", uuid::Uuid::new_v4());
        let capacity = Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_STEPS));
        let mut changes = super::signals::subscribe();
        loop {
            changes.borrow_and_update();
            let deadline = match dispatch(&app, &capacity, &executor_id, hooks).await {
                Ok(deadline) => deadline,
                Err(error) => {
                    log::error!("Space Run scheduler tick failed: {error}");
                    match crate::clock::unix_time_millis() {
                        Ok(now) => Some(now + ERROR_RETRY_MILLIS),
                        Err(error) => {
                            log::error!("Space Run scheduler clock failed: {error}");
                            return;
                        }
                    }
                }
            };
            if let Err(error) = scheduler::wakeup::wait(&mut changes, deadline).await {
                log::error!("Space Run scheduler stopped: {error}");
                return;
            }
        }
    });
}

async fn dispatch(
    app: &AppHandle,
    capacity: &Arc<tokio::sync::Semaphore>,
    executor_id: &str,
    hooks: SchedulerHooks,
) -> Result<Option<i64>, String> {
    maintain(app, hooks).await?;
    let pool = crate::spaces::db::pool(app).await?;
    while let Ok(permit) = capacity.clone().try_acquire_owned() {
        let Some(claim) = scheduler::claim_ready_step(pool, executor_id).await? else {
            break;
        };
        let registration = super::active_attempts::register(&claim)?;
        let worker_app = app.clone();
        tauri::async_runtime::spawn(async move {
            let _registration = registration;
            if let Err(error) = execute_claim(&worker_app, claim, hooks).await {
                log::error!("Space Run execution failed: {error}");
            }
            drop(permit);
            super::signals::announce();
        });
    }
    scheduler::wakeup::next_deadline(pool, crate::clock::unix_time_millis()?).await
}

async fn maintain(app: &AppHandle, hooks: SchedulerHooks) -> Result<(), String> {
    let pool = crate::spaces::db::pool(app).await?;
    let recovery_changed = scheduler::recover(pool).await?;
    let controls_changed = scheduler::apply_requested_controls(pool).await?;
    if let Err(error) =
        (hooks.after_controls)(app, pool, recovery_changed || controls_changed).await
    {
        log::error!("Run capability maintenance is pending recovery: {error}");
    }
    super::active_attempts::reconcile(pool).await?;
    scheduler::resume_waiting_steps(pool).await?;
    Ok(())
}

async fn execute_claim(
    app: &AppHandle,
    claim: super::AttemptClaim,
    hooks: SchedulerHooks,
) -> Result<(), AppError> {
    let pool = crate::spaces::db::pool(app).await?;
    let outcome = execute_attempt(app, pool, &claim, hooks).await;
    // Cleanup follows process/Attempt settlement on every return path. The feature checks
    // terminal authority, so a retryable or still-active Run retains its source snapshot.
    if let Err(error) = (hooks.after_attempt)(app, pool, &claim).await {
        log::error!(
            "Run capability cleanup is pending recovery for {}: {error}",
            claim.run_id
        );
    }
    outcome
}

async fn execute_attempt(
    app: &AppHandle,
    pool: &sqlx::SqlitePool,
    claim: &super::AttemptClaim,
    hooks: SchedulerHooks,
) -> Result<(), AppError> {
    super::active_attempts::reconcile(pool).await?;
    if claim.cancellation.is_cancelled() {
        return Err("run-stale: attempt was cancelled before execution".into());
    }
    let execution = (hooks.execute_step)(app, pool, claim);
    tokio::pin!(execution);
    let outcome = loop {
        tokio::select! {
            execution_outcome = &mut execution => break execution_outcome,
            _ = claim.cancellation.cancelled() => {
                // Local owners await process-tree termination before releasing compute permits.
                let _ = (&mut execution).await;
                return Err("cancelled: Run attempt was interrupted".into());
            }
            _ = tokio::time::sleep(Duration::from_secs(10)) => {
                if let Err(error) = scheduler::renew_lease(pool, claim).await {
                    claim.cancellation.cancel();
                    let _ = (&mut execution).await;
                    return Err(error.into());
                }
            }
        }
    };
    let execution_outcome = match outcome {
        Ok(StepOutcome::Completed {
            output,
            selected_branch,
        }) => {
            match scheduler::complete_step(pool, claim, &output, selected_branch.as_deref()).await {
                Err(error) if error.code == "budget_exhausted" => {
                    scheduler::fail_step(pool, claim, &error).await
                }
                completion => completion,
            }
        }
        Ok(StepOutcome::Waiting { reason, checkpoint }) => {
            scheduler::wait_step(pool, claim, reason, &checkpoint).await
        }
        Err(error) => scheduler::fail_step(pool, claim, &error).await,
    };
    super::signals::announce();
    execution_outcome
}
