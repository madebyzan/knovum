use std::path::Path;

use crate::ai::{
    contracts::UploadedFileReference,
    error::{AiError, AiErrorCode, AiResult},
    models::ExecutionTarget,
    providers::resolve_provider,
    transport::SecurityContext,
};

pub async fn upload_file_path(
    target: &ExecutionTarget,
    file_path: &Path,
) -> AiResult<UploadedFileReference> {
    if !file_path.is_file() {
        return Err(AiError::new(
            AiErrorCode::InvalidRequest,
            format!("File not found: {}", file_path.display()),
        ));
    }

    let file_bytes = tokio::fs::read(file_path).await.map_err(|e| {
        AiError::new(
            AiErrorCode::InvalidRequest,
            format!("Failed to read file: {e}"),
        )
    })?;

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
    target: &ExecutionTarget,
    bytes: Vec<u8>,
    mime_type: &str,
    file_name: &str,
) -> AiResult<UploadedFileReference> {
    let kind = target.model_provider_kind()?;
    let provider = resolve_provider(kind);
    let uploader = provider
        .as_file_upload()
        .ok_or_else(|| AiError::capability_not_supported(target.provider(), "file upload"))?;

    let ctx = SecurityContext::from_target(target)?;
    uploader
        .upload_ephemeral_file(&ctx, bytes, mime_type, file_name)
        .await
}
