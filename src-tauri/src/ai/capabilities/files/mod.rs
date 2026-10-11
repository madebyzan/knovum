use std::path::Path;
use tokio::io::AsyncReadExt;

pub(super) const MAX_UPLOAD_BYTES: usize = 32 * 1024 * 1024;

use crate::ai::{
    contracts::UploadedFileReference, models::ModelExecutionTarget, providers::resolve_provider,
    transport::SecurityContext,
};
use crate::foundation::app_error::{AppError, AppResult};

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

    let file_bytes = read_file_bounded(file_path, MAX_UPLOAD_BYTES).await?;

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
    if bytes.len() > MAX_UPLOAD_BYTES {
        return Err(AppError::invalid(
            "Reference file exceeds the 32 MiB upload limit",
        ));
    }
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

pub(super) async fn read_file_bounded(
    file_path: &Path,
    maximum_bytes: usize,
) -> AppResult<Vec<u8>> {
    let file = tokio::fs::File::open(file_path)
        .await
        .map_err(|error| AppError::invalid(format!("Failed to read file: {error}")))?;
    let mut bytes = Vec::new();
    file.take(maximum_bytes as u64 + 1)
        .read_to_end(&mut bytes)
        .await
        .map_err(|error| AppError::invalid(format!("Failed to read file: {error}")))?;
    if bytes.len() > maximum_bytes {
        return Err(AppError::invalid(
            "Reference file exceeds the 32 MiB upload limit",
        ));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests;
