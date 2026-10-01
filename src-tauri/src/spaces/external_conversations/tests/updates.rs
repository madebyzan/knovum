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
