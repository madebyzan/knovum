use crate::app_error::AppError;
use std::{sync::Arc, time::Duration};

use tauri::AppHandle;

use super::{scheduler, StepOutcome};

const ERROR_RETRY_MILLIS: i64 = 5_000;
const MAX_CONCURRENT_STEPS: usize = 4;

pub(crate) fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let executor_id = format!("desktop:{}", uuid::Uuid::new_v4());
        let capacity = Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_STEPS));
        let mut changes = super::signals::subscribe();
        loop {
            changes.borrow_and_update();
            let deadline = match dispatch(&app, &capacity, &executor_id).await {
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
) -> Result<Option<i64>, String> {
    maintain(app).await?;
    let pool = crate::spaces::db::pool(app).await?;
    while let Ok(permit) = capacity.clone().try_acquire_owned() {
        let Some(claim) = scheduler::claim_ready_step(pool, executor_id).await? else {
            break;
        };
        let registration = super::active_attempts::register(&claim)?;
        let worker_app = app.clone();
        tauri::async_runtime::spawn(async move {
            let _registration = registration;
            if let Err(error) = execute_claim(&worker_app, claim).await {
                log::error!("Space Run execution failed: {error}");
            }
            drop(permit);
            super::signals::announce();
        });
    }
    scheduler::wakeup::next_deadline(pool, crate::clock::unix_time_millis()?).await
}

async fn maintain(app: &AppHandle) -> Result<(), String> {
    let pool = crate::spaces::db::pool(app).await?;
    let recovery_changed = scheduler::recover(pool).await?;
    let controls_changed = scheduler::apply_requested_controls(pool).await?;
    if recovery_changed || controls_changed {
        if let Err(error) =
            crate::spaces::speech::voice_candidates::reconcile_terminal_snapshots(app, pool).await
        {
            log::error!("Terminal voice candidate snapshot cleanup is pending recovery: {error}");
        }
    }
    super::active_attempts::reconcile(pool).await?;
    scheduler::resume_waiting_steps(pool).await?;
    Ok(())
}

async fn execute_claim(app: &AppHandle, claim: super::AttemptClaim) -> Result<(), AppError> {
    let pool = crate::spaces::db::pool(app).await?;
    let outcome = execute_attempt(app, pool, &claim).await;
    // Cleanup follows process/Attempt settlement on every return path. The feature checks
    // terminal authority, so a retryable or still-active Run retains its source snapshot.
    if let Err(error) =
        crate::spaces::speech::voice_candidates::cleanup_terminal_snapshot(app, pool, &claim.run_id)
            .await
    {
        log::error!(
            "Terminal voice candidate snapshot cleanup is pending recovery for {}: {error}",
            claim.run_id
        );
    }
    outcome
}

async fn execute_attempt(
    app: &AppHandle,
    pool: &sqlx::SqlitePool,
    claim: &super::AttemptClaim,
) -> Result<(), AppError> {
    super::active_attempts::reconcile(pool).await?;
    if claim.cancellation.is_cancelled() {
        return Err("run-stale: attempt was cancelled before execution".into());
    }
    let execution = crate::spaces::workflows::execute_step(app, pool, claim);
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
                result => result,
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
