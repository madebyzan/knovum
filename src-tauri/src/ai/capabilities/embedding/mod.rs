use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use crate::ai::contracts::{
    embedding_wire::{EmbeddingExecutionIdentity, EmbeddingRole},
    EmbeddingCapability, EmbeddingResponse,
};

pub const EMBEDDING_SPEC_ID: &str =
    "qwen3-embedding-0.6b:28255eff53c370348c73c06e47af24a08692641967a32d1413bf18bb0867296e";

struct QwenEmbeddingSpec;

impl EmbeddingCapability for QwenEmbeddingSpec {
    fn model_id(&self) -> &'static str {
        "qwen3-embedding-0.6b"
    }

    fn dimensions(&self) -> usize {
        1_024
    }

    fn max_batch_size(&self) -> usize {
        32
    }

    fn max_chunk_bytes(&self) -> usize {
        1_900
    }
}

pub fn active_text_provider() -> &'static dyn EmbeddingCapability {
    &QwenEmbeddingSpec
}

pub async fn embed<R: Runtime>(
    app: &AppHandle<R>,
    texts: Vec<String>,
    role: EmbeddingRole,
    identity: &EmbeddingExecutionIdentity,
    resources: &crate::local_resources::preparation::PreparedModels,
    cancellation: &CancellationToken,
) -> Result<EmbeddingResponse, String> {
    let provider = active_text_provider();
    validate_inputs(&texts, provider)?;
    let expected = texts.len();
    let result = crate::ai::local_runtime::encode_embedding(
        app,
        texts,
        role,
        identity,
        resources,
        cancellation,
    )
    .await?;
    validate_vectors(&result.vectors, expected, provider.dimensions())?;
    crate::ai::usage::record(
        crate::ai::usage::UsagePurpose::KnowledgeIndexing,
        "local",
        provider.model_id(),
        &crate::ai::contracts::TokenUsage {
            prompt_tokens: result.input_tokens,
            completion_tokens: 0,
            total_tokens: result.input_tokens,
        },
    )
    .await
    .map_err(|error| error.to_string())?;
    Ok(EmbeddingResponse {
        vectors: result.vectors,
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
        return Err("invalid_output: embedding vectors have invalid shape or values".into());
    }
    Ok(())
}
