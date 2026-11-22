use std::{
    future::Future,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use crate::foundation::app_error::AppError;

#[derive(Clone, Copy, Debug)]
pub(super) struct TimeoutBudget {
    pub(super) idle: Duration,
    pub(super) max: Duration,
}

#[derive(Clone, Debug)]
pub(super) struct ActivityClock {
    last_activity: Arc<Mutex<Instant>>,
}

impl ActivityClock {
    pub(super) fn now() -> Self {
        Self {
            last_activity: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub(super) fn touch(&self) {
        if let Ok(mut guard) = self.last_activity.lock() {
            *guard = Instant::now();
        }
    }

    fn last(&self) -> Instant {
        self.last_activity
            .lock()
            .map(|guard| *guard)
            .unwrap_or_else(|poisoned| *poisoned.into_inner())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BudgetExceeded {
    Idle(Duration),
    Total(Duration),
}

impl BudgetExceeded {
    pub(super) fn into_error(self) -> AppError {
        match self {
            Self::Idle(duration) => AppError::new(
                "provider_request_failed",
                format!(
                    "Desktop Agent stopped responding for {}",
                    format_minutes(duration)
                ),
            ),
            Self::Total(duration) => AppError::new(
                "provider_request_failed",
                format!(
                    "Desktop Agent task exceeded its {} maximum time budget",
                    format_minutes(duration)
                ),
            ),
        }
    }
}

fn format_minutes(duration: Duration) -> String {
    let minutes = (duration.as_secs() / 60).max(1);
    if minutes == 1 {
        "1 minute".to_owned()
    } else {
        format!("{minutes} minutes")
    }
}

/// Awaits `execution` while enforcing both an idle-stall timeout (reset whenever `activity.touch()`
/// records progress) and an overall wall-clock ceiling.
pub(super) async fn await_with_budget<F, T>(
    execution: F,
    activity: ActivityClock,
    budget: TimeoutBudget,
    cancellation: &CancellationToken,
) -> Result<T, BudgetExceeded>
where
    F: Future<Output = T>,
{
    tokio::pin!(execution);
    let max_sleep = tokio::time::sleep(budget.max);
    tokio::pin!(max_sleep);
    let idle_sleep = tokio::time::sleep(budget.idle);
    tokio::pin!(idle_sleep);

    loop {
        tokio::select! {
            biased;
            outcome = &mut execution => return Ok(outcome),
            _ = &mut max_sleep => {
                cancellation.cancel();
                let _ = execution.await;
                return Err(BudgetExceeded::Total(budget.max));
            }
            _ = &mut idle_sleep => {
                let next_deadline = activity.last() + budget.idle;
                if next_deadline > Instant::now() {
                    idle_sleep.as_mut().reset(next_deadline);
                    continue;
                }
                cancellation.cancel();
                let _ = execution.await;
                return Err(BudgetExceeded::Idle(budget.idle));
            }
        }
    }
}

#[cfg(test)]
mod tests;
