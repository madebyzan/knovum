pub use knovum_inference_contracts::embedding_wire::{
    EmbeddingEncodeParams, EmbeddingEncoding, EmbeddingRole, EMBEDDING_SPEC_ID,
};

#[derive(Clone, Debug)]
pub struct EmbeddingExecutionIdentity {
    pub owner_kind: String,
    pub owner_id: String,
    pub attempt_id: String,
    pub attempt_epoch: u64,
    pub input_revision: String,
    /// Milliseconds the encode itself may take. Queueing for the inference slot and starting a
    /// cold worker are charged to neither this budget nor the caller.
    pub budget_ms: u64,
}
