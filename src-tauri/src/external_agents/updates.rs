use std::path::{Path, PathBuf};

use agent_client_protocol::{
    schema::v1::{AudioContent, ContentBlock, ImageContent, SessionNotification, SessionUpdate},
    util::MatchDispatch,
    Error, SessionMessage,
};
use base64::{engine::general_purpose::STANDARD, Engine};

pub enum AgentSessionUpdate {
    Thought(String),
    Message(String),
    ToolCall {
        id: String,
        title: String,
    },
    ToolUpdate {
        id: String,
    },
    PermissionRequested {
        request_id: String,
        title: String,
        options: Vec<AgentPermissionOption>,
    },
}

pub struct AgentPermissionOption {
    pub id: String,
    pub label: String,
    pub kind: &'static str,
}

pub(super) async fn receive_message(
    message: SessionMessage,
    workspace: &Path,
    on_update: &super::session::AgentUpdateHandler,
) -> Result<(), Error> {
    let SessionMessage::SessionMessage(dispatch) = message else {
        return Ok(());
    };
    MatchDispatch::new(dispatch)
        .if_notification(async |notification: SessionNotification| {
            if let Some(update) = receive(notification.update, workspace).await? {
                on_update(update)
                    .map_err(|error| Error::internal_error().data(error.to_string()))?;
            }
            Ok(())
        })
        .await
        .otherwise_ignore()
}

pub async fn receive(
    update: SessionUpdate,
    workspace: &Path,
) -> Result<Option<AgentSessionUpdate>, Error> {
    match update {
        SessionUpdate::AgentThoughtChunk(chunk) => match chunk.content {
            ContentBlock::Text(text) => Ok(Some(AgentSessionUpdate::Thought(text.text))),
            _ => Ok(None),
        },
        SessionUpdate::AgentMessageChunk(chunk) => match chunk.content {
            ContentBlock::Text(text) => Ok(Some(AgentSessionUpdate::Message(text.text))),
            ContentBlock::Image(image) => persist_image(workspace, image).await.map(|_| None),
            ContentBlock::Audio(audio) => persist_audio(workspace, audio).await.map(|_| None),
            _ => Ok(None),
        },
        SessionUpdate::ToolCall(call) => Ok(Some(AgentSessionUpdate::ToolCall {
            id: call.tool_call_id.0.to_string(),
            title: call.title,
        })),
        SessionUpdate::ToolCallUpdate(update) => Ok(Some(AgentSessionUpdate::ToolUpdate {
            id: update.tool_call_id.0.to_string(),
        })),
        _ => Ok(None),
    }
}

async fn persist_image(workspace: &Path, content: ImageContent) -> Result<PathBuf, Error> {
    persist_media(workspace, content.data, &content.mime_type, "image").await
}

async fn persist_audio(workspace: &Path, content: AudioContent) -> Result<PathBuf, Error> {
    persist_media(workspace, content.data, &content.mime_type, "audio").await
}

async fn persist_media(
    workspace: &Path,
    encoded: String,
    mime_type: &str,
    stem: &str,
) -> Result<PathBuf, Error> {
    let bytes = STANDARD.decode(encoded).map_err(|error| {
        Error::internal_error().data(format!("Agent returned invalid media: {error}"))
    })?;
    let extension = mime_guess::get_mime_extensions_str(mime_type)
        .and_then(|values| values.first())
        .ok_or_else(|| {
            Error::internal_error().data(format!("Agent returned unsupported media: {mime_type}"))
        })?;
    let path = workspace.join(format!("{stem}-{}.{}", uuid::Uuid::new_v4(), extension));
    tokio::fs::write(&path, bytes).await.map_err(|error| {
        Error::internal_error().data(format!("Cannot save Agent media: {error}"))
    })?;
    Ok(path)
}
