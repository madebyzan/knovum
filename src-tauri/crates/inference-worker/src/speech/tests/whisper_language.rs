use super::window;

#[test]
fn detection_starts_at_source_speech_and_uses_at_most_one_native_window() {
    let mut samples = vec![0.0; 16_000 * 95];
    samples[16_000 * 60] = 0.5;
    let speech = window(&samples, 60_000).unwrap();
    assert_eq!(speech[0], 0.5);
    assert_eq!(speech.len(), 16_000 * 30);
    assert_eq!(window(&samples, 94_000).unwrap().len(), 16_000);
}

#[test]
fn detection_rejects_empty_out_of_source_and_overflowing_offsets() {
    assert!(window(&[], 0).is_err());
    assert!(window(&[0.0; 16], 1).is_err());
    assert!(window(&[0.0; 16], u64::MAX).is_err());
}
