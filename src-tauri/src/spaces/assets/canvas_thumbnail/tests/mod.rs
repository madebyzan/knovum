use super::MAX_THUMBNAIL_BYTES;

#[test]
fn thumbnail_limit_is_bounded() {
    assert_eq!(MAX_THUMBNAIL_BYTES, 10 * 1024 * 1024);
}
