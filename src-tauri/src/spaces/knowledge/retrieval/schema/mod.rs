use std::sync::Arc;

use arrow_schema::{DataType, Field, Schema};

pub(crate) const TABLE_KNOWLEDGE_CHUNKS: &str = "knowledge_chunks";
pub(crate) const VECTOR_COLUMN: &str = "vector";
pub(crate) const MAX_SEMANTIC_LIMIT: i64 = 200;

#[derive(Debug, Clone)]
pub(crate) struct RetrievalRecord {
    pub chunk_id: String,
    pub space_id: String,
    pub node_id: String,
    pub node_kind: String,
    pub node_title: String,
    pub chunk_kind: String,
    pub source_revision: String,
    pub model: String,
    pub dimensions: i32,
    pub text: String,
    pub locator: String,
    pub metadata: String,
    pub vector: Vec<f32>,
    pub updated_at: i64,
}

pub(crate) fn schema(dimensions: i32) -> Schema {
    Schema::new(vec![
        Field::new("chunk_id", DataType::Utf8, false),
        Field::new("space_id", DataType::Utf8, false),
        Field::new("node_id", DataType::Utf8, false),
        Field::new("node_kind", DataType::Utf8, false),
        Field::new("node_title", DataType::Utf8, true),
        Field::new("chunk_kind", DataType::Utf8, false),
        Field::new("source_revision", DataType::Utf8, false),
        Field::new("model", DataType::Utf8, false),
        Field::new("dimensions", DataType::Int32, false),
        Field::new("text", DataType::Utf8, false),
        Field::new("locator", DataType::Utf8, false),
        Field::new("metadata", DataType::Utf8, false),
        Field::new(
            VECTOR_COLUMN,
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                dimensions,
            ),
            false,
        ),
        Field::new("updated_at", DataType::Int64, false),
    ])
}

pub(crate) fn validate_query_vector(vector: &[f32], limit: i64) -> Result<(), String> {
    if vector.is_empty() || vector.iter().any(|value| !value.is_finite()) {
        return Err("[SEMANTIC_QUERY_INVALID] query vector must contain finite values".into());
    }
    if vector.iter().map(|value| value * value).sum::<f32>() <= 0.0 {
        return Err("[SEMANTIC_QUERY_INVALID] query vector has zero norm".into());
    }
    if !(1..=MAX_SEMANTIC_LIMIT).contains(&limit) {
        return Err(format!(
            "Semantic result limit must be between 1 and {MAX_SEMANTIC_LIMIT}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
