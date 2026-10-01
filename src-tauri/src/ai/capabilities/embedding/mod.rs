use crate::ai::{
    contracts::{EmbeddingCapability, EmbeddingResponse},
    providers::local_candle::{self, LocalCandleEmbeddingAdapter},
};

pub fn active_text_provider() -> &'static dyn EmbeddingCapability {
    &LocalCandleEmbeddingAdapter
}

pub async fn embed(
    texts: Vec<String>,
    resources: &crate::local_resources::preparation::PreparedModels,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<EmbeddingResponse, String> {
    let provider = active_text_provider();
    validate_inputs(&texts, provider)?;
    let expected = texts.len();
    let output = local_candle::engine::embed(texts, resources.clone(), cancellation).await?;
    validate_vectors(&output.vectors, expected, provider.dimensions())?;
    let input_tokens = u32::try_from(output.input_tokens)
        .map_err(|_| "Embedding input token count exceeds the supported range".to_owned())?;
    crate::ai::usage::record(
        crate::ai::usage::UsagePurpose::KnowledgeIndexing,
        "local",
        provider.model_id(),
        &crate::ai::contracts::TokenUsage {
            prompt_tokens: input_tokens,
            completion_tokens: 0,
            total_tokens: input_tokens,
        },
    )
    .await
    .map_err(|error| error.to_string())?;
    Ok(EmbeddingResponse {
        vectors: output.vectors,
    })
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
