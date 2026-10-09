use super::{decode_reference_image, push_inline_reference, MAX_INLINE_REFERENCES_BYTES};

#[test]
fn rejects_oversized_inline_reference_before_decoding() {
    assert!(decode_reference_image("MTIzNDU=", 4).is_err());
}

#[test]
fn rejects_inline_references_that_exceed_the_total_budget() {
    let mut prepared = Vec::new();
    let mut total_bytes = MAX_INLINE_REFERENCES_BYTES - 1;
    assert!(push_inline_reference(&mut prepared, &mut total_bytes, "xx".into()).is_err());
    assert!(prepared.is_empty());
}
