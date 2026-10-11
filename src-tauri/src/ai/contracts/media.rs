use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::ai::transport::SecurityContext;
use crate::foundation::app_error::AppResult;

#[derive(Debug, Clone)]
pub struct ImageGenerationInput {
    pub prompt: String,
    pub aspect_ratio: String,
}

#[derive(Debug, Clone)]
pub struct GeneratedImage {
    pub bytes: Vec<u8>,
    pub provider: String,
    pub model: String,
    pub revised_prompt: Option<String>,
}

pub trait ImageGenerationCapability: Send + Sync {
    fn generate<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a ImageGenerationInput,
    ) -> BoxFuture<'a, AppResult<GeneratedImage>>;
}

#[derive(Debug, Clone)]
pub struct VideoGenerationInput {
    pub prompt: String,
    pub ratio: String,
    pub duration: u8,
    pub reference_images: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoGenerationSubmission {
    pub operation_id: String,
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum VideoGenerationStatus {
    Pending { progress_pct: Option<u8> },
    Processing { progress_pct: Option<u8> },
    Succeeded { video_url: String },
    Failed { error_message: String },
}

#[derive(Debug, Clone)]
pub struct GeneratedVideo {
    pub bytes: Vec<u8>,
    pub provider: String,
    pub model: String,
    pub operation_id: String,
}

pub type VideoGenerationStageCallback<'a> =
    Box<dyn Fn(&str) -> Result<(), String> + Send + Sync + 'a>;

pub trait VideoGenerationCapability: Send + Sync {
    fn submit<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a VideoGenerationInput,
    ) -> BoxFuture<'a, AppResult<VideoGenerationSubmission>>;

    fn status<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        operation_id: &'a str,
    ) -> BoxFuture<'a, AppResult<VideoGenerationStatus>>;

    fn generate<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        req: &'a VideoGenerationInput,
        cancellation: &'a CancellationToken,
        stage: VideoGenerationStageCallback<'a>,
    ) -> BoxFuture<'a, AppResult<GeneratedVideo>>;
}
