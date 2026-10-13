use super::*;
use crate::ai::local_runtime::device_profiles::MemorySnapshot;

const GIB: u64 = 1024 * 1024 * 1024;

#[test]
fn reservation_accepts_exact_available_and_inference_budget_boundaries() {
    let snapshot = MemorySnapshot {
        total_bytes: 16 * GIB,
        available_bytes: 5 * GIB,
    };
    assert_eq!(
        validate_memory_reservation(4 * GIB, "test", snapshot, 0).unwrap(),
        4 * GIB
    );
    assert_eq!(
        validate_memory_reservation(2 * GIB, "test", snapshot, 8 * GIB).unwrap(),
        10 * GIB
    );
}

#[test]
fn reservation_rejects_missing_headroom_and_exceeded_inference_budget() {
    let low_available = MemorySnapshot {
        total_bytes: 16 * GIB,
        available_bytes: 5 * GIB - 1,
    };
    assert!(
        validate_memory_reservation(4 * GIB, "test", low_available, 0)
            .unwrap_err()
            .starts_with("resource_exhausted:")
    );

    let over_budget = MemorySnapshot {
        total_bytes: 16 * GIB,
        available_bytes: 8 * GIB,
    };
    assert!(
        validate_memory_reservation(4 * GIB, "test", over_budget, 8 * GIB)
            .unwrap_err()
            .contains("inference budget")
    );
}

#[test]
fn reservation_rejects_arithmetic_overflow() {
    let snapshot = MemorySnapshot {
        total_bytes: u64::MAX,
        available_bytes: u64::MAX,
    };
    assert!(validate_memory_reservation(u64::MAX, "test", snapshot, 0)
        .unwrap_err()
        .contains("requirement overflowed"));
}

fn isolated() -> Arc<Admission> {
    Arc::new(Admission {
        state: Mutex::new(State::default()),
    })
}

/// A background request behind a long holder must still run. Failing the waiter instead is what
/// turned one slow transcription into an indexing failure for as long as the user kept working.
#[tokio::test]
async fn a_waiter_outlasts_a_holder_that_runs_far_longer_than_any_request_budget() {
    let admission = isolated();
    let idle = CancellationToken::new();
    let holder = admission
        .enqueue(Priority::Background, &idle, &idle)
        .await
        .expect("the first request owns the idle slot");
    let waiting = tokio::spawn({
        let admission = admission.clone();
        let idle = idle.clone();
        async move { admission.enqueue(Priority::Background, &idle, &idle).await }
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(!waiting.is_finished(), "the slot is still held");
    drop(holder);
    waiting
        .await
        .expect("the waiter task completes")
        .expect("releasing the slot grants it to the waiter");
}

/// Giving up is the caller's decision, so cancellation is the only way out of the queue.
#[tokio::test]
async fn cancelling_a_waiter_leaves_the_queue_able_to_grant_the_slot() {
    let admission = isolated();
    let idle = CancellationToken::new();
    let holder = admission
        .enqueue(Priority::Interactive, &idle, &idle)
        .await
        .expect("the first request owns the idle slot");
    let abandoned = CancellationToken::new();
    let cancelled = tokio::spawn({
        let admission = admission.clone();
        let abandoned = abandoned.clone();
        let idle = idle.clone();
        async move {
            admission
                .enqueue(Priority::Interactive, &abandoned, &idle)
                .await
        }
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    abandoned.cancel();
    assert_eq!(
        cancelled
            .await
            .expect("the cancelled task completes")
            .unwrap_err(),
        "cancelled"
    );
    drop(holder);
    admission
        .enqueue(Priority::Interactive, &idle, &idle)
        .await
        .expect("the abandoned waiter left no claim on the slot");
}
