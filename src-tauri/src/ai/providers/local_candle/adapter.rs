use crate::ai::contracts::EmbeddingCapability;

pub struct LocalCandleEmbeddingAdapter;

const QUERY_INSTRUCTION: &str =
    "Given a knowledge-space search query, retrieve relevant passages from the user's documents, notes, media transcripts, code, and web captures.";

impl LocalCandleEmbeddingAdapter {
    pub const MODEL_ID: &'static str = "qwen3-embedding-0.6b";
    pub const DIMENSIONS: usize = 1_024;
    pub const MAX_BATCH_SIZE: usize = 32;
}

impl EmbeddingCapability for LocalCandleEmbeddingAdapter {
    fn model_id(&self) -> &'static str {
        Self::MODEL_ID
    }

    fn dimensions(&self) -> usize {
        Self::DIMENSIONS
    }

    fn max_batch_size(&self) -> usize {
        Self::MAX_BATCH_SIZE
    }

    fn format_query(&self, query: &str) -> String {
        format!("Instruct: {QUERY_INSTRUCTION}\nQuery:{query}")
    }
}
