use std::sync::Arc;

use arrow_schema::{DataType, Field, Schema};

pub(crate) const TABLE_KNOWLEDGE_CHUNKS: &str = "knowledge_chunks";
pub(crate) const VECTOR_COLUMN: &str = "vector";

#[derive(Debug, Clone)]
pub(crate) struct IndexedKnowledgeChunk {
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
                Arc::new(Field::new("value", DataType::Float32, true)),
                dimensions,
            ),
            false,
        ),
        Field::new("updated_at", DataType::Int64, false),
    ])
}

#[cfg(test)]
mod tests;
