use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::ai::transport::SecurityContext;
use crate::foundation::app_error::AppResult;

#[derive(Debug, Clone)]
pub struct ImageGenerationInput {
    pub prompt: String,
    /// The frame the user asked for. `None` when nobody asked - changing an
    /// attached picture keeps the picture's frame. An adapter reads it on its
    /// text-to-image path; on a reference request it applies it only if its API
    /// has a frame field.
    pub aspect_ratio: Option<String>,
    /// Pixels the model must see. Empty for a pure text-to-image request.
    /// What to do with them is the prompt's job, not a separate kind of request.
    pub references: Vec<ImageReference>,
}

impl ImageGenerationInput {
    /// A text-to-image request always carries a frame; the shared layer checks
    /// that before an adapter is reached.
    pub fn require_aspect_ratio(&self) -> AppResult<&str> {
        self.aspect_ratio.as_deref().ok_or_else(|| {
            crate::foundation::app_error::AppError::new(
                "invalid_request",
                "A text-to-image request needs an aspect ratio",
            )
        })
    }
}

/// A source rendered as the user currently sees it. References always travel as
/// PNG, because that is what the Space renders from an asset and its edit.
#[derive(Debug, Clone)]
pub struct ImageReference {
    pub png: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// What an adapter accepts alongside the prompt, for one configured model.
/// Declared by the adapter, never assumed by the caller.
#[derive(Debug, Clone, Copy)]
pub struct ImageReferenceSupport {
    pub max_images: u8,
    pub max_bytes_per_image: usize,
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
        cancellation: &'a CancellationToken,
    ) -> BoxFuture<'a, AppResult<GeneratedImage>>;

    /// None when the configured model only accepts a text prompt.
    fn reference_support(&self, _provider: &str, _model: &str) -> Option<ImageReferenceSupport> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct VideoGenerationInput {
    pub prompt: String,
    pub ratio: String,
    pub duration_seconds: u8,
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
