use super::{persist_media, persist_resource, resource_stem};
use agent_client_protocol::{
    schema::v1::{ContentBlock, ToolCallContent, ToolCallStatus},
    Error,
};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub(super) async fn persist_tool_content(
    workspace: &Path,
    tool_id: &str,
    content: &[ToolCallContent],
) -> Result<(), Error> {
    let prefix = tool_artifact_prefix(tool_id);
    let mut active_paths = Vec::new();
    let mut local_paths = Vec::new();
    for (index, item) in content.iter().enumerate() {
        if let ToolCallContent::Content(block) = item {
            let identity = format!("{prefix}{index}");
            if let Some(output) =
                persist_content(workspace, block.content.clone(), Some(&identity)).await?
            {
                if output.local_reference {
                    local_paths.push(output.path.clone());
                }
                active_paths.push(output.path);
            }
        }
    }
    let obsolete = crate::external_agents::artifacts::obsolete_tool_artifacts(
        workspace,
        tool_id,
        &active_paths,
    )
    .map_err(projection_error)?;
    for path in obsolete {
        tokio::fs::remove_file(path)
            .await
            .map_err(|error| Error::internal_error().data(error.to_string()))?;
    }
    crate::external_agents::artifacts::record_tool(workspace, tool_id, &active_paths, &local_paths)
        .map_err(projection_error)?;
    Ok(())
}

pub(super) struct PersistedOutput {
    pub(super) path: PathBuf,
    pub(super) local_reference: bool,
}

pub(super) async fn persist_content(
    workspace: &Path,
    content: ContentBlock,
    identity: Option<&str>,
) -> Result<Option<PersistedOutput>, Error> {
    match content {
        ContentBlock::Text(_) => Ok(None),
        ContentBlock::Image(image) => {
            let stem = image
                .uri
                .as_deref()
                .and_then(resource_stem)
                .unwrap_or_else(|| "image".into());
            persist_media(workspace, image.data, &image.mime_type, &stem, identity)
                .await
                .map(|path| {
                    Some(PersistedOutput {
                        path,
                        local_reference: false,
                    })
                })
        }
        ContentBlock::Audio(audio) => {
            persist_media(workspace, audio.data, &audio.mime_type, "audio", identity)
                .await
                .map(|path| {
                    Some(PersistedOutput {
                        path,
                        local_reference: false,
                    })
                })
        }
        ContentBlock::Resource(resource) => persist_resource(workspace, resource, identity)
            .await
            .map(|path| {
                Some(PersistedOutput {
                    path,
                    local_reference: false,
                })
            }),
        ContentBlock::ResourceLink(link) => {
            let local_reference =
                tauri::Url::parse(&link.uri).is_ok_and(|url| url.scheme() == "file");
            crate::external_agents::resource_links::materialize(workspace, link, identity)
                .await
                .map(|path| {
                    path.map(|path| PersistedOutput {
                        path,
                        local_reference,
                    })
                })
        }
        _ => Err(Error::invalid_request().data("Unsupported ACP output content")),
    }
}

pub(super) fn projection_error(error: crate::foundation::app_error::AppError) -> Error {
    Error::invalid_request().data(error.to_string())
}

pub(super) fn tool_artifact_prefix(tool_id: &str) -> String {
    let identity = Sha256::digest(tool_id.as_bytes());
    format!("tool-{identity}-", identity = hex::encode(identity))
}

pub(super) fn tool_status(status: ToolCallStatus) -> Result<&'static str, Error> {
    match status {
        ToolCallStatus::Pending => Ok("pending"),
        ToolCallStatus::InProgress => Ok("in_progress"),
        ToolCallStatus::Completed => Ok("completed"),
        ToolCallStatus::Failed => Ok("failed"),
        _ => Err(Error::invalid_request().data("Unsupported ACP tool status")),
    }
}

pub(super) fn tool_context(
    content: &[ToolCallContent],
    raw: Option<&serde_json::Value>,
) -> Option<String> {
    content
        .iter()
        .find_map(|item| match item {
            ToolCallContent::Content(block) => match &block.content {
                ContentBlock::Text(text) => Some(compact(&text.text)),
                _ => None,
            },
            ToolCallContent::Diff(diff) => Some(diff.path.display().to_string()),
            _ => None,
        })
        .or_else(|| raw.map(compact_value))
}

pub(super) fn compact_value(value: &serde_json::Value) -> String {
    compact(&value.to_string())
}

fn compact(value: &str) -> String {
    let mut chars = value.chars();
    let excerpt: String = chars.by_ref().take(4_000).collect();
    if chars.next().is_some() {
        format!("{excerpt}…")
    } else {
        excerpt
    }
}
