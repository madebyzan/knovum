use tauri::{AppHandle, Runtime};

use crate::ai::{
    contracts::{EmbeddingCapability, EmbeddingResponse},
    providers::local_candle::{self, LocalCandleEmbeddingAdapter},
};

pub use crate::ai::contracts::EmbeddingCapability as TextEmbeddingProvider;

pub fn active_text_provider() -> &'static dyn EmbeddingCapability {
    &LocalCandleEmbeddingAdapter
}

pub fn unload() -> Result<(), String> {
    local_candle::engine::unload()
}

pub async fn embed<R: Runtime>(
    app: &AppHandle<R>,
    texts: Vec<String>,
) -> Result<EmbeddingResponse, String> {
    let provider = active_text_provider();
    validate_inputs(&texts, provider)?;
    let expected = texts.len();
    let vectors = local_candle::engine::embed(app.clone(), texts).await?;
    validate_vectors(&vectors, expected, provider.dimensions())?;
    Ok(EmbeddingResponse { vectors })
}

fn validate_inputs(texts: &[String], provider: &dyn EmbeddingCapability) -> Result<(), String> {
    let max_batch = provider.max_batch_size();
    if texts.is_empty() || texts.len() > max_batch {
        return Err(format!(
            "Embedding requests require between 1 and {max_batch} texts"
        ));
    }
    let max_bytes = provider.max_chunk_bytes();
    if texts
        .iter()
        .any(|text| text.trim().is_empty() || text.len() > max_bytes)
    {
        return Err(format!(
            "Embedding texts must be non-empty and under {max_bytes} bytes"
        ));
    }
    Ok(())
}

fn validate_vectors(
    vectors: &[Vec<f32>],
    expected: usize,
    dimensions: usize,
) -> Result<(), String> {
    if vectors.len() != expected
        || vectors.iter().any(|vector| {
            vector.len() != dimensions || vector.iter().any(|value| !value.is_finite())
        })
    {
        return Err("Local embedding model produced malformed or non-finite vectors".into());
    }
    Ok(())
}
