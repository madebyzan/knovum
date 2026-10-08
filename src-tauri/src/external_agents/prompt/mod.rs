use std::path::PathBuf;

use agent_client_protocol::schema::v1::{ContentBlock, ResourceLink, TextContent};
use tauri::{AppHandle, Runtime};

use crate::app_error::{AppError, AppResult};

mod attachments;

use attachments::AgentAttachments;

#[cfg(test)]
mod tests;

pub struct AgentPrompt {
    text: String,
    attachments: AgentAttachments,
}

impl AgentPrompt {
    pub(crate) fn from_authorized_assets(
        text: String,
        attachments: Vec<PathBuf>,
        authorize: impl Fn(&std::path::Path) -> AppResult<()>,
    ) -> AppResult<Self> {
        AgentAttachments::from_authorized_assets(attachments, authorize)
            .map(|attachments| Self { text, attachments })
    }

    pub fn from_renderer<R: Runtime>(
        app: &AppHandle<R>,
        text: String,
        attachments: Vec<PathBuf>,
    ) -> AppResult<Self> {
        AgentAttachments::from_renderer(app, attachments)
            .map(|attachments| Self { text, attachments })
    }

    pub async fn into_content(self) -> AppResult<Vec<ContentBlock>> {
        let mut content = vec![ContentBlock::Text(TextContent::new(self.text))];
        for path in self.attachments.into_paths() {
            content.push(ContentBlock::Text(TextContent::new(format!(
                "Attached local source (read-only input): {}. Inspect this file with your available tools; save deliverables in the output workspace.",
                path.display()
            ))));
            content.push(attachment(path).await?);
        }
        Ok(content)
    }
}

async fn attachment(path: PathBuf) -> AppResult<ContentBlock> {
    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|error| AppError::invalid(format!("Cannot inspect attachment: {error}")))?;
    if !metadata.is_file() {
        return Err(AppError::invalid("Agent attachments must be regular files"));
    }
    let uri = tauri::Url::from_file_path(&path)
        .map_err(|()| AppError::invalid("Attachment path cannot be represented as a file URL"))?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| AppError::invalid("Attachment filename is not valid Unicode"))?;
    let size = i64::try_from(metadata.len())
        .map_err(|_| AppError::invalid("Attachment is too large for ACP metadata"))?;
    Ok(ContentBlock::ResourceLink(
        ResourceLink::new(name, uri.to_string())
            .mime_type(mime.to_string())
            .size(size),
    ))
}
