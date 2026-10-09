use std::path::Path;

use crate::ai::contracts::AgentArtifactKind;
use crate::app_error::{AppError, AppResult};

pub(super) fn classify(path: &Path) -> AppResult<AgentArtifactKind> {
    if let Some(detected) =
        infer::get_from_path(path).map_err(|error| AppError::internal(error.to_string()))?
    {
        let mime = detected.mime_type();
        let kind = match mime.split_once('/').map(|value| value.0) {
            Some("image") => AgentArtifactKind::Image,
            Some("video") => AgentArtifactKind::Video,
            Some("audio") => AgentArtifactKind::Audio,
            _ if mime == "application/pdf" || mime.starts_with("text/") => {
                AgentArtifactKind::Document
            }
            _ => AgentArtifactKind::File,
        };
        return Ok(kind);
    }
    let kind = match path.extension().and_then(|value| value.to_str()) {
        Some("docx" | "pptx" | "xlsx") => AgentArtifactKind::Document,
        _ => AgentArtifactKind::File,
    };
    Ok(kind)
}
