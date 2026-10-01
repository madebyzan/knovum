use super::*;

#[tokio::test]
async fn cleanup_waits_for_its_writer_without_blocking_other_runs() {
    let id = uuid::Uuid::new_v4().to_string();
    let writer = acquire(&id).await.unwrap();
    let (started, waiting) = tokio::sync::oneshot::channel();
    let cleanup = tokio::spawn(async move {
        started.send(()).unwrap();
        acquire(&id).await.unwrap()
    });
    waiting.await.unwrap();
    let other = acquire(&uuid::Uuid::new_v4().to_string()).await.unwrap();
    assert!(!cleanup.is_finished());
    drop(writer);
    cleanup.await.unwrap();
    drop(other);
}
