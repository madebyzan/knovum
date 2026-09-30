use super::ModelExecutionTarget;

#[derive(Debug)]
pub struct ImageAnalysisRequest {
    pub target: ModelExecutionTarget,
    pub source: std::path::PathBuf,
    pub instruction: String,
    pub system_prompt: String,
    pub max_tokens: Option<u32>,
}
