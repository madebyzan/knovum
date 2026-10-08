use super::*;
use crate::app_error::AppResult;
use std::sync::atomic::{AtomicBool, Ordering};

fn operation() -> (String, PluginOperation) {
    let id = uuid::Uuid::new_v4().to_string();
    let operation =
        PluginOperation::begin(&id, Channel::new(|_| Ok(())), &CancellationToken::new()).unwrap();
    (id, operation)
}

#[tokio::test]
async fn cancellation_prevents_source_admission_and_commit() {
    let (id, operation) = operation();
    let read = AtomicBool::new(false);
    cancel_plugin_operation(id).unwrap();
    let result = operation
        .read(async {
            read.store(true, Ordering::SeqCst);
            Ok::<(), AppError>(())
        })
        .await;
    let error = result.unwrap_err();
    assert_eq!(error.code, "cancelled");
    assert_eq!(error.message, "Plugin operation was cancelled");
    assert!(!read.load(Ordering::SeqCst));
    assert_eq!(operation.commit().unwrap_err().code, "cancelled");
}

#[test]
fn commit_admission_rejects_late_cancellation() {
    let (id, operation) = operation();
    operation.commit().unwrap();
    assert_eq!(
        cancel_plugin_operation(id).unwrap_err().code,
        "plugin_operation_committing"
    );
    assert!(!operation.cancellation().is_cancelled());
}

#[test]
fn operation_drop_releases_registration_and_interrupts_background_reads() {
    let (id, operation) = operation();
    let cancellation = operation.cancellation().clone();
    drop(operation);
    assert!(cancellation.is_cancelled());
    assert!(registry().get(&id).is_err());
}

#[tokio::test]
async fn owner_shutdown_cancels_pending_reading_and_prevents_commit() {
    let parent = CancellationToken::new();
    let operation = PluginOperation::begin(
        &uuid::Uuid::new_v4().to_string(),
        Channel::new(|_| Ok(())),
        &parent,
    )
    .unwrap();
    parent.cancel();
    assert_eq!(
        operation
            .read(std::future::pending::<AppResult<()>>())
            .await
            .unwrap_err()
            .code,
        "cancelled"
    );
    assert_eq!(operation.commit().unwrap_err().code, "cancelled");
}
