use super::checked_region;

#[test]
fn vad_centiseconds_preserve_source_offset_and_bound_final_padding() {
    assert_eq!(checked_region(6000.0, 6395.4, 63954).unwrap(), 60000..63954);
    assert_eq!(checked_region(1.0, 401.0, 4000).unwrap(), 10..4000);
}

#[test]
fn vad_rejects_nonfinite_reversed_and_out_of_source_intervals() {
    for (start, end) in [
        (f32::NAN, 10.0),
        (0.0, f32::INFINITY),
        (-1.0, 10.0),
        (10.0, 9.0),
        (0.0, 410.0),
        (400.0, 401.0),
    ] {
        assert!(checked_region(start, end, 4000).is_err());
    }
}
