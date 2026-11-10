use std::path::Path;
use tokio::io::AsyncReadExt;

pub(super) const MAX_UPLOAD_BYTES: usize = 32 * 1024 * 1024;

use crate::foundation::app_error::{AppError, AppResult};

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
