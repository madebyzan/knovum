use super::{best_detection_start, window};

#[test]
fn detection_starts_at_source_speech_and_uses_at_most_one_inference_window() {
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

#[test]
fn best_detection_start_prefers_substantial_speech_regions() {
    let regions = vec![500..1_000, 2_000..6_000, 7_000..9_000];
    assert_eq!(best_detection_start(&regions), 2_000);

    let short_regions = vec![500..1_000, 1_200..1_800];
    assert_eq!(best_detection_start(&short_regions), 1_200);

    assert_eq!(best_detection_start(&[]), 0);
}
