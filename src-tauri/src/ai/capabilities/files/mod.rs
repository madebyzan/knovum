use std::path::Path;

use crate::ai::{
    contracts::UploadedFileReference, models::ModelExecutionTarget, providers::resolve_provider,
    transport::SecurityContext,
};
use crate::app_error::{AppError, AppResult};

pub async fn upload_file_path(
    target: &ModelExecutionTarget,
    file_path: &Path,
) -> AppResult<UploadedFileReference> {
    if !file_path.is_file() {
        return Err(AppError::new(
            "invalid_request",
            format!("File not found: {}", file_path.display()),
        ));
    }

    let file_bytes = tokio::fs::read(file_path)
        .await
        .map_err(|e| AppError::new("invalid_request", format!("Failed to read file: {e}")))?;

    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("upload.bin");

    let mime_type = mime_guess::from_path(file_path)
        .first_or_octet_stream()
        .to_string();

    upload_file_bytes(target, file_bytes, &mime_type, file_name).await
}

pub async fn upload_file_bytes(
    target: &ModelExecutionTarget,
    bytes: Vec<u8>,
    mime_type: &str,
    file_name: &str,
) -> AppResult<UploadedFileReference> {
    let kind = target.model_provider_kind()?;
    let provider = resolve_provider(kind);
    let uploader = provider
        .as_file_upload()
        .ok_or_else(|| AppError::capability_not_supported(target.provider(), "file upload"))?;

    let ctx = SecurityContext::from_target(target).await?;
    uploader
        .upload_ephemeral_file(&ctx, bytes, mime_type, file_name)
        .await
}
