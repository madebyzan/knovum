pub use knovum_inference_contracts::embedding_wire::*;

#[derive(Clone, Debug)]
pub struct EmbeddingExecutionIdentity {
    pub owner_kind: String,
    pub owner_id: String,
    pub attempt_id: String,
    pub attempt_epoch: u64,
    pub input_revision: String,
    pub deadline_ms: u64,
}
