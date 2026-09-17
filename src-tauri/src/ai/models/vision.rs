use super::ExecutionTarget;

#[derive(Debug)]
pub struct ImageAnalysisRequest {
    pub target: ExecutionTarget,
    pub image_path: String,
    pub instruction: String,
    pub system_prompt: String,
    pub max_tokens: Option<u32>,
}
