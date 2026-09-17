use super::trim_incomplete_utf8_tail;

#[test]
fn byte_truncation_does_not_invent_a_replacement_character() {
    let mut bytes = "hello😀".as_bytes()[..7].to_vec();
    trim_incomplete_utf8_tail(&mut bytes);
    assert_eq!(String::from_utf8(bytes).unwrap(), "hello");

    let mut invalid_then_truncated = vec![0xFF];
    invalid_then_truncated.extend_from_slice(&"😀".as_bytes()[..2]);
    trim_incomplete_utf8_tail(&mut invalid_then_truncated);
    assert_eq!(invalid_then_truncated, vec![0xFF]);
}
