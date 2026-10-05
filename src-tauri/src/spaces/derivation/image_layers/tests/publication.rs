use super::Generation;

#[test]
fn accepted_cancellation_prevents_publication() {
    let generation = Generation::default();
    generation.cancel().unwrap();
    assert_eq!(
        generation.begin_publication().unwrap_err().code,
        "cancelled"
    );
}

#[test]
fn publication_admission_rejects_late_cancellation() {
    let generation = Generation::default();
    generation.begin_publication().unwrap();
    assert!(generation.cancel().is_err());
    assert!(!generation.cancellation().is_cancelled());
    assert!(generation.begin_publication().is_err());
}

#[test]
fn concurrent_cancellation_and_publication_have_one_winner() {
    for _ in 0..32 {
        let generation = std::sync::Arc::new(Generation::default());
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let cancelling = generation.clone();
        let cancel_barrier = barrier.clone();
        let cancelled = std::thread::spawn(move || {
            cancel_barrier.wait();
            cancelling.cancel()
        });
        barrier.wait();
        let published = generation.begin_publication();
        assert_ne!(cancelled.join().unwrap().is_ok(), published.is_ok());
    }
}
