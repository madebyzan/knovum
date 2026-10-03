#[derive(Debug, Clone)]
pub struct EmbeddingResponse {
    pub vectors: Vec<Vec<f32>>,
}

pub trait EmbeddingCapability: Send + Sync {
    fn model_id(&self) -> &'static str;
    fn dimensions(&self) -> usize;
    fn max_batch_size(&self) -> usize;
    fn max_chunk_bytes(&self) -> usize {
        64 * 1024
    }
}
