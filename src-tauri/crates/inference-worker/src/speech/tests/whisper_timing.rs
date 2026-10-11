use super::time;

#[test]
fn whisper_zero_padding_is_not_published_as_source_audio() {
    assert_eq!(time(580, 91_744).unwrap(), 5_734);
    assert_eq!(time(570, 91_744).unwrap(), 5_700);
    assert_eq!(time(3_573, 91_744).unwrap(), 5_734);
}

#[test]
fn timestamps_outside_the_inference_padded_context_are_rejected() {
    for timestamp in [-1, 3_574, i64::MAX] {
        assert!(time(timestamp, 91_744).is_err());
    }
}
