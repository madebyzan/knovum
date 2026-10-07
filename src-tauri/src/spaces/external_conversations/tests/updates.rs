use super::{enqueue_update, AgentSessionUpdate, MAX_UPDATE_TEXT_BYTES};

#[test]
fn update_queue_rejects_a_burst_above_its_capacity() {
    let (sender, _receiver) = tokio::sync::mpsc::channel(1);
    assert!(enqueue_update(&sender, AgentSessionUpdate::Message("first".into())).is_ok());
    assert!(enqueue_update(&sender, AgentSessionUpdate::Message("second".into())).is_err());
    assert!(enqueue_update(&sender, AgentSessionUpdate::Thought("ignored".into())).is_ok());
}

#[test]
fn update_queue_rejects_an_oversized_message() {
    let (sender, _receiver) = tokio::sync::mpsc::channel(1);
    assert!(enqueue_update(
        &sender,
        AgentSessionUpdate::Message("x".repeat(MAX_UPDATE_TEXT_BYTES + 1))
    )
    .is_err());
}

#[tokio::test]
async fn an_accepted_flush_survives_session_registry_release() {
    let id = uuid::Uuid::new_v4().to_string();
    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
    super::active_updates()
        .lock()
        .unwrap()
        .insert(id.clone(), sender);
    let flush = super::flush(&id);
    super::release(&id);
    let worker = tokio::spawn(async move {
        let Some(super::UpdateCommand::Flush(acknowledge)) = receiver.recv().await else {
            panic!("accepted flush must keep the update queue alive");
        };
        acknowledge.send(Ok(())).unwrap();
    });
    flush.await.unwrap();
    worker.await.unwrap();
    assert!(super::flush(&id).await.is_err());
}
