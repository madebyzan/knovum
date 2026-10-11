use super::super::credits::Credits;
use crate::cancellation::Cancellation;
use knovum_inference_contracts::runtime::RuntimeAckChunkParams;
use std::sync::{mpsc, Arc};
use std::time::Duration;

#[test]
fn ninth_chunk_waits_for_matching_ack_and_stale_epoch_is_rejected() {
    let credits = Arc::new(Credits::new("request", 2));
    let cancel = Cancellation::new(u64::MAX);
    for seq in 1..=8 {
        credits.reserve(seq, 32, &cancel).unwrap();
    }
    let (send, receive) = mpsc::channel();
    let current = credits.clone();
    let running = cancel.clone();
    let thread = std::thread::spawn(move || send.send(current.reserve(9, 32, &running)).unwrap());
    assert!(receive.recv_timeout(Duration::from_millis(150)).is_err());
    credits
        .acknowledge(&RuntimeAckChunkParams {
            request_id: "request".into(),
            attempt_epoch: 2,
            seq: 1,
            consumed_bytes: 32,
        })
        .unwrap();
    receive
        .recv_timeout(Duration::from_secs(2))
        .unwrap()
        .unwrap();
    thread.join().unwrap();
    assert!(credits
        .acknowledge(&RuntimeAckChunkParams {
            request_id: "request".into(),
            attempt_epoch: 1,
            seq: 2,
            consumed_bytes: 32
        })
        .is_err());
    assert!(credits.wait(&cancel).is_err());
}

#[test]
fn cancellation_wakes_waiters_without_freeing_unconsumed_credits() {
    let credits = Arc::new(Credits::new("request", 2));
    let cancel = Cancellation::new(u64::MAX);
    credits.reserve(1, 16 * 1024 * 1024, &cancel).unwrap();
    let current = credits.clone();
    let running = cancel.clone();
    let thread = std::thread::spawn(move || current.reserve(2, 4, &running));
    cancel.cancel();
    assert_eq!(thread.join().unwrap().unwrap_err(), "cancelled");
}
