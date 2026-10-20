use std::collections::HashMap;

use arrow_array::{FixedSizeListArray, Float32Array, RecordBatch, StringArray};

use crate::spaces::knowledge::retrieval::schema::VECTOR_COLUMN;

pub(super) fn centroid(vectors: &[Vec<f32>]) -> Result<Vec<f32>, String> {
    let dimensions = vectors[0].len();
    let mut mean_embedding = vec![0.0f32; dimensions];
    for vector in vectors {
        if vector.len() != dimensions {
            return Err("Source embeddings have inconsistent dimensions".to_owned());
        }
        for (sum, value) in mean_embedding.iter_mut().zip(vector) {
            *sum += value;
        }
    }
    let norm = mean_embedding
        .iter()
        .map(|value| value * value)
        .sum::<f32>()
        .sqrt();
    if !norm.is_finite() || norm <= 0.0 {
        return Err("Source embeddings cannot form a valid related-content query".to_owned());
    }
    for value in &mut mean_embedding {
        *value /= norm;
    }
    Ok(mean_embedding)
}

pub(super) fn source_vectors(
    batches: &[RecordBatch],
    revision_by_chunk_id: &HashMap<String, String>,
) -> Result<Vec<Vec<f32>>, String> {
    let mut vectors = Vec::new();
    for batch in batches {
        let ids = batch
            .column_by_name("chunk_id")
            .and_then(|column| column.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| "Source embeddings are missing chunk IDs".to_owned())?;
        let revisions = batch
            .column_by_name("source_revision")
            .and_then(|column| column.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| "Source embeddings are missing revisions".to_owned())?;
        let column = batch
            .column_by_name(VECTOR_COLUMN)
            .and_then(|column| column.as_any().downcast_ref::<FixedSizeListArray>())
            .ok_or_else(|| "Source embeddings are missing vectors".to_owned())?;
        for row in 0..batch.num_rows() {
            if !revision_by_chunk_id
                .get(ids.value(row))
                .is_some_and(|revision| revision == revisions.value(row))
            {
                continue;
            }
            let values = column.value(row);
            let floats = values
                .as_any()
                .downcast_ref::<Float32Array>()
                .ok_or_else(|| "Source embedding has an unexpected vector type".to_owned())?;
            vectors.push(floats.values().to_vec());
        }
    }
    Ok(vectors)
}
