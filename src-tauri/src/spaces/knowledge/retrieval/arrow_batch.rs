use std::sync::Arc;

use arrow_array::{
    ArrayRef, FixedSizeListArray, Float32Array, Int32Array, Int64Array, RecordBatch, StringArray,
};
use arrow_schema::{DataType, Field};

use super::schema::{schema, IndexedKnowledgeChunk};

/// Encodes retrieval records as a LanceDB `RecordBatch`.
///
/// All records must share the same vector dimensionality; mixed dimensions are rejected.
pub(super) fn records_to_batch(
    records: &[IndexedKnowledgeChunk],
    dimensions: i32,
) -> Result<RecordBatch, String> {
    if records.iter().any(|record| record.dimensions != dimensions) {
        return Err("Embedding batch contains mixed dimensions".into());
    }
    let vector_array = build_vector_array(records, dimensions)?;
    RecordBatch::try_new(
        Arc::new(schema(dimensions)),
        build_columns(records, vector_array),
    )
    .map_err(|error| format!("Failed to build LanceDB record batch: {error}"))
}

fn build_vector_array(
    records: &[IndexedKnowledgeChunk],
    dimensions: i32,
) -> Result<FixedSizeListArray, String> {
    let vectors = records
        .iter()
        .flat_map(|record| record.vector.iter().copied())
        .collect::<Vec<f32>>();
    let vector_values = Float32Array::from(vectors);
    let field = Arc::new(Field::new("value", DataType::Float32, true));
    FixedSizeListArray::try_new(field, dimensions, Arc::new(vector_values), None)
        .map_err(|error| format!("Failed to build LanceDB vector array: {error}"))
}

fn build_columns(
    records: &[IndexedKnowledgeChunk],
    vector_array: FixedSizeListArray,
) -> Vec<ArrayRef> {
    vec![
        string_column(records, |r| r.chunk_id.as_str()),
        string_column(records, |r| r.space_id.as_str()),
        string_column(records, |r| r.node_id.as_str()),
        string_column(records, |r| r.node_kind.as_str()),
        string_column(records, |r| r.node_title.as_str()),
        string_column(records, |r| r.chunk_kind.as_str()),
        string_column(records, |r| r.source_revision.as_str()),
        string_column(records, |r| r.model.as_str()),
        Arc::new(Int32Array::from(
            records.iter().map(|r| r.dimensions).collect::<Vec<_>>(),
        )) as ArrayRef,
        string_column(records, |r| r.text.as_str()),
        string_column(records, |r| r.locator.as_str()),
        string_column(records, |r| r.metadata.as_str()),
        Arc::new(vector_array) as ArrayRef,
        Arc::new(Int64Array::from(
            records.iter().map(|r| r.updated_at).collect::<Vec<_>>(),
        )) as ArrayRef,
    ]
}

fn string_column<F>(records: &[IndexedKnowledgeChunk], select: F) -> ArrayRef
where
    F: Fn(&IndexedKnowledgeChunk) -> &str,
{
    Arc::new(StringArray::from(
        records.iter().map(select).collect::<Vec<_>>(),
    )) as ArrayRef
}
