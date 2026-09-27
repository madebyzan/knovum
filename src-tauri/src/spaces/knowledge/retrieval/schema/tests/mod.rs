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
