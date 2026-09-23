use std::path::PathBuf;

use agent_client_protocol::schema::v1::{
    AudioContent, ContentBlock, ImageContent, PromptCapabilities, ResourceLink, TextContent,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use tauri::{AppHandle, Runtime};

use crate::ai::error::{AiError, AiResult};

mod attachments;

use attachments::AgentAttachments;

const CONTEXT_DIRECTORY: &str = ".knovum-context";

pub struct AgentPrompt {
    text: String,
    attachments: AgentAttachments,
}

impl AgentPrompt {
    pub fn from_renderer<R: Runtime>(
        app: &AppHandle<R>,
        text: String,
        attachments: Vec<PathBuf>,
    ) -> AiResult<Self> {
        AgentAttachments::from_renderer(app, attachments)
            .map(|attachments| Self { text, attachments })
    }

    pub async fn into_content(
        self,
        working_directory: &std::path::Path,
        capabilities: PromptCapabilities,
    ) -> AiResult<Vec<ContentBlock>> {
        let mut content = vec![ContentBlock::Text(TextContent::new(self.text))];
        let context_directory = working_directory.join(CONTEXT_DIRECTORY);
        if !self.attachments.is_empty() {
            tokio::fs::create_dir(&context_directory)
                .await
                .map_err(|error| {
                    AiError::internal(format!("Cannot prepare Agent context: {error}"))
                })?;
        }
        for (index, source) in self.attachments.into_paths().into_iter().enumerate() {
            let path = attachments::stage(source, &context_directory, index).await?;
            content.push(attachment(path, &capabilities).await?);
        }
        Ok(content)
    }
}

pub async fn discard_staged_context(working_directory: &std::path::Path) -> AiResult<()> {
    let directory = working_directory.join(CONTEXT_DIRECTORY);
    if tokio::fs::try_exists(&directory)
        .await
        .map_err(|error| AiError::internal(format!("Cannot inspect Agent context: {error}")))?
    {
        tokio::fs::remove_dir_all(directory)
            .await
            .map_err(|error| AiError::internal(format!("Cannot remove Agent context: {error}")))?;
    }
    Ok(())
}

async fn attachment(path: PathBuf, capabilities: &PromptCapabilities) -> AiResult<ContentBlock> {
    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    if mime.type_() == mime::IMAGE && capabilities.image {
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|error| AiError::invalid(format!("Cannot read image attachment: {error}")))?;
        return Ok(ContentBlock::Image(ImageContent::new(
            STANDARD.encode(bytes),
            mime.to_string(),
        )));
    }
    if mime.type_() == mime::AUDIO && capabilities.audio {
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|error| AiError::invalid(format!("Cannot read audio attachment: {error}")))?;
        return Ok(ContentBlock::Audio(AudioContent::new(
            STANDARD.encode(bytes),
            mime.to_string(),
        )));
    }
    let metadata = path
        .metadata()
        .map_err(|error| AiError::invalid(format!("Cannot inspect attachment: {error}")))?;
    let uri = tauri::Url::from_file_path(&path)
        .map_err(|()| AiError::invalid("Attachment path cannot be represented as a file URL"))?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| AiError::invalid("Attachment filename is not valid Unicode"))?;
    let size = i64::try_from(metadata.len())
        .map_err(|_| AiError::invalid("Attachment is too large for ACP metadata"))?;
    Ok(ContentBlock::ResourceLink(
        ResourceLink::new(name, uri.to_string())
            .mime_type(mime.to_string())
            .size(size),
    ))
}
