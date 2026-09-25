use std::{sync::Arc, time::Duration};

use tauri::AppHandle;

use super::{scheduler, StepOutcome};

const IDLE_INTERVAL: Duration = Duration::from_millis(350);
const MAX_CONCURRENT_STEPS: usize = 4;

pub(crate) fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let executor_id = format!("desktop:{}", uuid::Uuid::new_v4());
        let capacity = Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_STEPS));
        loop {
            if let Err(error) = maintain(&app).await {
                log::error!("Space Run scheduler tick failed: {error}");
            }
            while let Ok(permit) = capacity.clone().try_acquire_owned() {
                let pool = match crate::spaces::db::pool(&app).await {
                    Ok(pool) => pool,
                    Err(error) => {
                        log::error!("Space Run pool unavailable: {error}");
                        break;
                    }
                };
                let claim = match scheduler::claim_ready_step(pool, &executor_id).await {
                    Ok(Some(claim)) => claim,
                    Ok(None) => break,
                    Err(error) => {
                        log::error!("Space Run claim failed: {error}");
                        break;
                    }
                };
                let worker_app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = execute_claim(&worker_app, claim).await {
                        log::error!("Space Run execution failed: {error}");
                    }
                    drop(permit);
                });
            }
            tokio::time::sleep(IDLE_INTERVAL).await;
        }
    });
}

async fn maintain(app: &AppHandle) -> Result<(), String> {
    let pool = crate::spaces::db::pool(app).await?;
    scheduler::recover(pool).await?;
    scheduler::apply_requested_controls(pool).await?;
    scheduler::resume_waiting_steps(pool).await?;
    Ok(())
}

async fn execute_claim(app: &AppHandle, claim: super::AttemptClaim) -> Result<(), String> {
    let pool = crate::spaces::db::pool(app).await?;
    let execution = crate::spaces::workflows::execute_step(app, pool, &claim);
    tokio::pin!(execution);
    let outcome = loop {
        tokio::select! {
            result = &mut execution => break result,
            _ = tokio::time::sleep(Duration::from_secs(10)) => {
                scheduler::renew_lease(pool, &claim).await?;
            }
        }
    };
    let result = match outcome {
        Ok(StepOutcome::Completed {
            output,
            selected_branch,
        }) => {
            let output_bytes = serde_json::to_vec(&output)
                .map_err(crate::spaces::db::json_error)?
                .len();
            if let Err(error) = super::reserve_output_bytes(pool, &claim.run_id, output_bytes).await
            {
                return scheduler::fail_step(pool, &claim, &error).await;
            }
            scheduler::complete_step(pool, &claim, &output, selected_branch.as_deref()).await
        }
        Ok(StepOutcome::Waiting { reason, checkpoint }) => {
            scheduler::wait_step(pool, &claim, reason, &checkpoint).await
        }
        Err(error) => scheduler::fail_step(pool, &claim, &error).await,
    };
    super::signals::announce();
    result
}
