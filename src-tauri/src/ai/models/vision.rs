use super::ExecutionTarget;

#[derive(Debug)]
pub struct ImageAnalysisRequest {
    pub target: ExecutionTarget,
    pub source: ImageAnalysisSource,
    pub instruction: String,
    pub system_prompt: String,
    pub max_tokens: Option<u32>,
}

#[derive(Debug)]
pub enum ImageAnalysisSource {
    FilePath(String),
    Png(Vec<u8>),
}
