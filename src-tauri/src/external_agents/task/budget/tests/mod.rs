use super::*;

#[tokio::test(start_paused = true)]
async fn activity_resets_idle_timeout_so_long_active_task_completes() {
    let activity = ActivityClock::now();
    let worker_activity = activity.clone();
    let cancellation = CancellationToken::new();
    let budget = TimeoutBudget {
        idle: Duration::from_secs(180),
        max: Duration::from_secs(900),
    };

    let outcome = await_with_budget(
        async move {
            for _ in 0..5 {
                tokio::time::sleep(Duration::from_secs(60)).await;
                worker_activity.touch();
            }
            "done"
        },
        activity,
        budget,
        &cancellation,
    )
    .await;

    assert_eq!(outcome, Ok("done"));
    assert!(!cancellation.is_cancelled());
}

#[tokio::test(start_paused = true)]
async fn stalled_task_fails_after_idle_timeout() {
    let activity = ActivityClock::now();
    let cancellation = CancellationToken::new();
    let worker_cancel = cancellation.clone();
    let budget = TimeoutBudget {
        idle: Duration::from_secs(180),
        max: Duration::from_secs(900),
    };

    let outcome = await_with_budget(
        async move {
            worker_cancel.cancelled().await;
        },
        activity,
        budget,
        &cancellation,
    )
    .await;

    assert_eq!(outcome, Err(BudgetExceeded::Idle(Duration::from_secs(180))));
    assert!(cancellation.is_cancelled());
}

#[tokio::test(start_paused = true)]
async fn active_task_still_stops_at_max_budget() {
    let activity = ActivityClock::now();
    let worker_activity = activity.clone();
    let cancellation = CancellationToken::new();
    let worker_cancel = cancellation.clone();
    let budget = TimeoutBudget {
        idle: Duration::from_secs(180),
        max: Duration::from_secs(900),
    };

    let outcome = await_with_budget(
        async move {
            loop {
                tokio::select! {
                    _ = worker_cancel.cancelled() => break,
                    _ = tokio::time::sleep(Duration::from_secs(60)) => {
                        worker_activity.touch();
                    }
                }
            }
        },
        activity,
        budget,
        &cancellation,
    )
    .await;

    assert_eq!(
        outcome,
        Err(BudgetExceeded::Total(Duration::from_secs(900)))
    );
    assert!(cancellation.is_cancelled());
}
