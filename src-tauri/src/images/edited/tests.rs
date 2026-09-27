use super::validate_source_dimensions;

#[test]
fn source_limits_reject_empty_and_oversized_images() {
    assert!(validate_source_dimensions((1, 1)).is_ok());
    for size in [(0, 1), (1, 0), (10_001, 1), (1, 10_001)] {
        assert!(validate_source_dimensions(size).is_err());
    }
    assert!(validate_source_dimensions((10_000, 10_000)).is_ok());
}
