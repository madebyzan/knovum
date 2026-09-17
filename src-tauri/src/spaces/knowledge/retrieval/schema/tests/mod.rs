use super::*;

#[test]
fn schema_contains_vector_column_with_expected_dimensions() {
    let dims = 384;
    let s = schema(dims);
    let field = s.field_with_name(VECTOR_COLUMN).expect("vector column");
    match field.data_type() {
        DataType::FixedSizeList(inner, size) => {
            assert_eq!(*size, dims);
            assert_eq!(inner.data_type(), &DataType::Float32);
        }
        other => panic!("expected FixedSizeList, got {other:?}"),
    }
}

#[test]
fn validate_query_vector_rejects_bad_input() {
    assert!(validate_query_vector(&[], 5).is_err());
    assert!(validate_query_vector(&[0.0, 0.0], 5).is_err());
    assert!(validate_query_vector(&[f32::NAN, 1.0], 5).is_err());
    assert!(validate_query_vector(&[1.0, 0.0], 0).is_err());
    assert!(validate_query_vector(&[1.0, 0.0], MAX_SEMANTIC_LIMIT + 1).is_err());
}

#[test]
fn validate_query_vector_accepts_reasonable_input() {
    assert!(validate_query_vector(&[0.1, 0.2, 0.3], 10).is_ok());
    assert!(validate_query_vector(&[1.0], MAX_SEMANTIC_LIMIT).is_ok());
}
