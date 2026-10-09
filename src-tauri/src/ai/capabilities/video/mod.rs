use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{GeneratedVideo, VideoGenerationInput},
    models::ModelExecutionTarget,
    providers::resolve_provider,
    transport::SecurityContext,
};
use crate::app_error::{AppError, AppResult};

pub mod materialize;
mod output;
pub use materialize::materialize;

const MAX_INLINE_REFERENCES_BYTES: usize = 64 * 1024 * 1024;

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
    let mut inline_bytes = 0;
    let provider = resolve_provider(target.model_provider_kind()?);
    let can_upload = provider.as_file_upload().is_some();
    for item in images {
        if item.starts_with("http://")
            || item.starts_with("https://")
            || item.starts_with("dashscope://")
        {
            prepared_list.push(item);
        } else if let Some(stripped_data) = item.strip_prefix("data:") {
            let (mime_part, base64_payload) = stripped_data
                .split_once(";base64,")
                .ok_or_else(|| AppError::invalid("Reference image data URI is invalid"))?;
            let decoded_bytes =
                decode_reference_data(base64_payload, super::files::MAX_UPLOAD_BYTES)?;
            if can_upload {
                let upload_ref = super::files::upload_file_bytes(
                    target,
                    decoded_bytes,
                    mime_part,
                    "reference_image.png",
                )
                .await?;
                prepared_list.push(upload_ref.access_url.unwrap_or(upload_ref.file_id));
            } else {
                push_inline_reference(&mut prepared_list, &mut inline_bytes, item)?;
            }
        } else {
            let path = std::path::Path::new(&item);
            if path.is_file() {
                if can_upload {
                    let upload_ref = super::files::upload_file_path(target, path).await?;
                    let effective_reference = upload_ref.access_url.unwrap_or(upload_ref.file_id);
                    prepared_list.push(effective_reference);
                } else {
                    let bytes =
                        super::files::read_file_bounded(path, super::files::MAX_UPLOAD_BYTES)
                            .await?;
                    let mime = mime_guess::from_path(path)
                        .first_or_octet_stream()
                        .to_string();
                    use base64::Engine;
                    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
                    push_inline_reference(
                        &mut prepared_list,
                        &mut inline_bytes,
                        format!("data:{mime};base64,{encoded}"),
                    )?;
                }
            } else {
                return Err(AppError::invalid("Reference image path is unavailable"));
            }
        }
    }
    Ok(prepared_list)
}

fn push_inline_reference(
    prepared: &mut Vec<String>,
    total_bytes: &mut usize,
    value: String,
) -> AppResult<()> {
    *total_bytes = total_bytes
        .checked_add(value.len())
        .filter(|total| *total <= MAX_INLINE_REFERENCES_BYTES)
        .ok_or_else(|| AppError::invalid("Inline reference images exceed the 64 MiB budget"))?;
    prepared.push(value);
    Ok(())
}

fn decode_reference_data(encoded: &str, maximum_bytes: usize) -> AppResult<Vec<u8>> {
    use base64::Engine;
    if encoded.len() > maximum_bytes.div_ceil(3).saturating_mul(4) {
        return Err(AppError::invalid(
            "Reference image exceeds the 32 MiB upload limit",
        ));
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|error| AppError::invalid(format!("Invalid reference image: {error}")))?;
    if bytes.len() > maximum_bytes {
        return Err(AppError::invalid(
            "Reference image exceeds the 32 MiB upload limit",
        ));
    }
    Ok(bytes)
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

#[cfg(test)]
mod tests;
