use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD, Engine};

use crate::ai::error::{AiError, AiResult};

use super::workspace::TaskWorkspace;

pub(super) async fn materialize(
    workspace: &TaskWorkspace,
    references: Vec<String>,
) -> AiResult<Vec<PathBuf>> {
    let directory = workspace.path().join(".knovum-inputs");
    if references.iter().any(|value| value.starts_with("data:")) {
        tokio::fs::create_dir(&directory)
            .await
            .map_err(|error| AiError::internal(error.to_string()))?;
    }
    let mut paths = Vec::with_capacity(references.len());
    for (index, reference) in references.into_iter().enumerate() {
        paths.push(materialize_one(&directory, index, reference).await?);
    }
    Ok(paths)
}

async fn materialize_one(directory: &Path, index: usize, reference: String) -> AiResult<PathBuf> {
    let Some(data) = reference.strip_prefix("data:") else {
        let path = PathBuf::from(reference);
        if !path.is_file() {
            return Err(AiError::invalid(
                "Desktop Agent media references must be local files or image data URLs",
            ));
        }
        return Ok(path);
    };
    let (declared, encoded) = data
        .split_once(";base64,")
        .ok_or_else(|| AiError::invalid("Media reference data URL is invalid"))?;
    let bytes = STANDARD.decode(encoded).map_err(|error| {
        AiError::invalid(format!("Media reference is not valid base64: {error}"))
    })?;
    let detected = infer::get(&bytes)
        .filter(|kind| kind.mime_type().starts_with("image/"))
        .ok_or_else(|| AiError::invalid("Media reference is not a supported image"))?;
    if detected.mime_type() != declared {
        return Err(AiError::invalid(
            "Media reference bytes do not match the declared type",
        ));
    }
    let path = directory.join(format!("reference-{index:03}.{}", detected.extension()));
    tokio::fs::write(&path, bytes)
        .await
        .map_err(|error| AiError::internal(error.to_string()))?;
    Ok(path)
}
