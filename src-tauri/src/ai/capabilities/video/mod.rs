use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::VideoGenerationInput, models::ModelExecutionTarget, providers::resolve_provider,
    transport::SecurityContext,
};
use crate::app_error::{AppError, AppResult};

pub mod materialize;
pub use crate::ai::contracts::GeneratedVideo;
pub use materialize::materialize;

#[derive(Debug)]
pub struct VideoGenerationRequest {
    pub target: ModelExecutionTarget,
    pub prompt: String,
    pub ratio: String,
    pub duration: u8,
    pub reference_images: Vec<String>,
}

pub async fn generate(
    _app: &AppHandle,
    request: VideoGenerationRequest,
    cancellation: &CancellationToken,
    stage: impl Fn(&str) -> Result<(), String> + Send + Sync + 'static,
) -> AppResult<GeneratedVideo> {
    validate(&request)?;
    let kind = request.target.model_provider_kind()?;
    let provider = resolve_provider(kind);
    let video_generation = provider.as_video_generation().ok_or_else(|| {
        AppError::capability_not_supported(request.target.provider(), "video generation")
    })?;

    let prepared_images =
        prepare_reference_images(&request.target, request.reference_images).await?;
    let ctx = SecurityContext::from_target(&request.target).await?;
    let req = VideoGenerationInput {
        prompt: request.prompt,
        ratio: request.ratio,
        duration: request.duration,
        reference_images: prepared_images,
    };

    video_generation
        .generate(&ctx, &req, cancellation, Box::new(stage))
        .await
}

async fn prepare_reference_images(
    target: &ModelExecutionTarget,
    images: Vec<String>,
) -> AppResult<Vec<String>> {
    let mut prepared_list = Vec::with_capacity(images.len());
    for item in images {
        if item.starts_with("http://")
            || item.starts_with("https://")
            || item.starts_with("dashscope://")
        {
            prepared_list.push(item);
        } else if let Some(stripped_data) = item.strip_prefix("data:") {
            let kind = target.model_provider_kind()?;
            let provider = resolve_provider(kind);
            if provider.as_file_upload().is_some() {
                if let Some((mime_part, base64_payload)) = stripped_data.split_once(";base64,") {
                    use base64::Engine;
                    let decoded_bytes = base64::engine::general_purpose::STANDARD
                        .decode(base64_payload)
                        .map_err(|e| AppError::new("invalid_request", e.to_string()))?;
                    let upload_ref = super::files::upload_file_bytes(
                        target,
                        decoded_bytes,
                        mime_part,
                        "reference_image.png",
                    )
                    .await?;
                    let effective_reference = upload_ref.access_url.unwrap_or(upload_ref.file_id);
                    prepared_list.push(effective_reference);
                    continue;
                }
            }
            prepared_list.push(item);
        } else {
            let path = std::path::Path::new(&item);
            if path.is_file() {
                let kind = target.model_provider_kind()?;
                let provider = resolve_provider(kind);
                if provider.as_file_upload().is_some() {
                    let upload_ref = super::files::upload_file_path(target, path).await?;
                    let effective_reference = upload_ref.access_url.unwrap_or(upload_ref.file_id);
                    prepared_list.push(effective_reference);
                } else {
                    let bytes = tokio::fs::read(path).await.map_err(|e| {
                        AppError::new("invalid_request", format!("Failed to read file: {e}"))
                    })?;
                    let mime = mime_guess::from_path(path)
                        .first_or_octet_stream()
                        .to_string();
                    use base64::Engine;
                    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
                    prepared_list.push(format!("data:{mime};base64,{encoded}"));
                }
            } else {
                prepared_list.push(item);
            }
        }
    }
    Ok(prepared_list)
}

fn validate(request: &VideoGenerationRequest) -> AppResult<()> {
    if request.prompt.trim().is_empty() || request.ratio.trim().is_empty() || request.duration == 0
    {
        return Err(AppError::new(
            "invalid_request",
            "Video generation request is invalid",
        ));
    }
    Ok(())
}
