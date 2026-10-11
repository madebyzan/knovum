use super::super::resume::parse_range;

#[test]
fn range_requires_a_bounded_numeric_interval() {
    assert_eq!(parse_range("bytes 2-3/4"), Some((2, 3, 4)));
    for invalid in [
        "bytes */4",
        "bytes 3-2/4",
        "bytes 0-4/4",
        "items 2-3/4",
        "bytes 2-3/*",
    ] {
        assert_eq!(parse_range(invalid), None, "{invalid}");
    }
}
