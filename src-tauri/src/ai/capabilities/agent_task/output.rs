use std::fs;

use tokio::io::AsyncReadExt;

use crate::ai::{
    capabilities::agent_artifacts,
    error::{AiError, AiErrorCode, AiResult},
};

use super::workspace::TaskWorkspace;

pub(super) enum ArtifactType {
    Image,
    Mp4Video,
}

pub(super) async fn read_unique(
    workspace: &TaskWorkspace,
    expected: ArtifactType,
) -> AiResult<Vec<u8>> {
    let artifacts = agent_artifacts::collect(workspace.path())?;
    let mut matched = None;
    let mut match_count = 0;
    for artifact in artifacts {
        let path = workspace.path().join(artifact.relative_path);
        if !expected.accepts_path(&path) {
            continue;
        }
        let bytes = read_verified(workspace.path(), &path).await?;
        if matches_type(&bytes, &expected) {
            match_count += 1;
            if match_count > 1 {
                break;
            }
            matched = Some(bytes);
        }
    }
    if match_count != 1 {
        return Err(AiError::new(
            AiErrorCode::InvalidResponse,
            format!(
                "Desktop Agent must produce exactly one {}, but produced {}",
                expected.label(),
                match_count
            ),
        ));
    }
    matched.ok_or_else(|| AiError::internal("Desktop Agent artifact selection is unavailable"))
}

async fn read_verified(root: &std::path::Path, path: &std::path::Path) -> AiResult<Vec<u8>> {
    let root = root
        .canonicalize()
        .map_err(|error| AiError::internal(error.to_string()))?;
    let file = fs::File::open(path).map_err(|error| AiError::invalid(error.to_string()))?;
    let current = path
        .canonicalize()
        .map_err(|error| AiError::invalid(error.to_string()))?;
    if !current.starts_with(&root) {
        return Err(AiError::invalid(
            "Desktop Agent artifact is outside its workspace",
        ));
    }
    let metadata = file
        .metadata()
        .map_err(|error| AiError::invalid(error.to_string()))?;
    let opened = same_file::Handle::from_file(
        file.try_clone()
            .map_err(|error| AiError::invalid(error.to_string()))?,
    )
    .map_err(|error| AiError::invalid(error.to_string()))?;
    let current_handle = same_file::Handle::from_path(&current)
        .map_err(|error| AiError::invalid(error.to_string()))?;
    if !metadata.is_file() || opened != current_handle {
        return Err(AiError::invalid(
            "Desktop Agent artifact changed before it was read",
        ));
    }
    let mut bytes = Vec::new();
    tokio::fs::File::from_std(file)
        .read_to_end(&mut bytes)
        .await
        .map_err(|error| AiError::internal(error.to_string()))?;
    Ok(bytes)
}

fn matches_type(bytes: &[u8], expected: &ArtifactType) -> bool {
    infer::get(bytes).is_some_and(|kind| match expected {
        ArtifactType::Image => {
            matches!(kind.mime_type(), "image/png" | "image/jpeg" | "image/webp")
        }
        ArtifactType::Mp4Video => kind.mime_type() == "video/mp4",
    })
}

impl ArtifactType {
    fn label(&self) -> &'static str {
        match self {
            Self::Image => "PNG, JPEG, or WebP image",
            Self::Mp4Video => "MP4 video",
        }
    }

    fn accepts_path(&self, path: &std::path::Path) -> bool {
        let extension = path.extension().and_then(|value| value.to_str());
        match self {
            Self::Image => matches!(extension, Some("png" | "jpg" | "jpeg" | "webp")),
            Self::Mp4Video => extension == Some("mp4"),
        }
    }
}
