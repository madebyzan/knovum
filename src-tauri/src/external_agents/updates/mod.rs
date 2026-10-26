use content_projection::{
    compact_value, persist_content, persist_tool_content, projection_error, tool_context,
    tool_status,
};
mod content_projection;
use std::path::{Path, PathBuf};

use agent_client_protocol::{
    schema::v1::{
        ContentBlock, EmbeddedResource, EmbeddedResourceResource, SessionNotification,
        SessionUpdate,
    },
    util::MatchDispatch,
    Error, SessionMessage,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

const MAX_EMBEDDED_RESOURCE_BYTES: usize = 32 * 1024 * 1024;
const MAX_EMBEDDED_IMAGE_BYTES: usize = 64 * 1024 * 1024;
const MAX_EMBEDDED_AUDIO_BYTES: usize = 128 * 1024 * 1024;

#[derive(Clone)]
pub enum AgentSessionUpdate {
    Message(String),
    ToolCall {
        id: String,
        title: String,
        status: String,
        context: Option<String>,
    },
    ToolUpdate {
        id: String,
        title: Option<String>,
        status: Option<String>,
        context: Option<String>,
    },
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
        // An Agent's reasoning is its own: it is neither shown nor recorded, so it stops here.
        SessionUpdate::AgentThoughtChunk(_) => Ok(None),
        SessionUpdate::AgentMessageChunk(chunk) => match chunk.content {
            ContentBlock::Text(text) => Ok(Some(AgentSessionUpdate::Message(text.text))),
            content => {
                let identity = message_occurrence_identity(
                    chunk
                        .message_id
                        .as_ref()
                        .map(|message_id| message_id.0.as_ref()),
                );
                if let Some(output) = persist_content(workspace, content, Some(&identity)).await? {
                    crate::external_agents::artifacts::record_message(
                        workspace,
                        &identity,
                        &output.path,
                        output.local_reference,
                    )
                    .map_err(projection_error)?;
                }
                Ok(None)
            }
        },
        SessionUpdate::ToolCall(call) => {
            persist_tool_content(workspace, &call.tool_call_id.0, &call.content).await?;
            Ok(Some(AgentSessionUpdate::ToolCall {
                id: call.tool_call_id.0.to_string(),
                title: call.title,
                status: tool_status(call.status)?.to_owned(),
                context: tool_context(
                    &call.content,
                    call.raw_output.as_ref().or(call.raw_input.as_ref()),
                ),
            }))
        }
        SessionUpdate::ToolCallUpdate(update) => {
            if let Some(content) = &update.fields.content {
                persist_tool_content(workspace, &update.tool_call_id.0, content).await?;
            }
            Ok(Some(AgentSessionUpdate::ToolUpdate {
                id: update.tool_call_id.0.to_string(),
                title: update.fields.title,
                status: update
                    .fields
                    .status
                    .map(tool_status)
                    .transpose()?
                    .map(str::to_owned),
                context: update
                    .fields
                    .content
                    .as_deref()
                    .and_then(|content| {
                        tool_context(
                            content,
                            update
                                .fields
                                .raw_output
                                .as_ref()
                                .or(update.fields.raw_input.as_ref()),
                        )
                    })
                    .or_else(|| {
                        update
                            .fields
                            .raw_output
                            .as_ref()
                            .or(update.fields.raw_input.as_ref())
                            .map(compact_value)
                    }),
            }))
        }
        _ => Ok(None),
    }
}

fn message_occurrence_identity(message_id: Option<&str>) -> String {
    let message = Sha256::digest(message_id.unwrap_or("unidentified").as_bytes());
    format!(
        "message-{message}-{}",
        uuid::Uuid::new_v4(),
        message = hex::encode(message)
    )
}

async fn persist_resource(
    workspace: &Path,
    content: EmbeddedResource,
    output_identity: Option<&str>,
) -> Result<PathBuf, Error> {
    let (bytes, mime_type, uri, default_extension) = match content.resource {
        EmbeddedResourceResource::TextResourceContents(text) => {
            if text.text.len() > MAX_EMBEDDED_RESOURCE_BYTES {
                return Err(size_limit_error("resource"));
            }
            (text.text.into_bytes(), text.mime_type, text.uri, "txt")
        }
        EmbeddedResourceResource::BlobResourceContents(blob) => (
            decode_bounded(blob.blob, MAX_EMBEDDED_RESOURCE_BYTES, "resource")?,
            blob.mime_type,
            blob.uri,
            "bin",
        ),
        _ => return Err(Error::invalid_request().data("Unsupported ACP embedded resource")),
    };
    let extensions = mime_type
        .as_deref()
        .and_then(|value| value.parse::<mime::Mime>().ok())
        .and_then(|value| mime_guess::get_mime_extensions_str(value.essence_str()));
    let uri_extension = Path::new(&uri).extension().and_then(|value| value.to_str());
    let extension = uri_extension
        .filter(|candidate| match extensions {
            Some(values) => values.contains(candidate),
            None => {
                candidate.len() <= 10
                    && candidate
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric())
            }
        })
        .or_else(|| extensions.and_then(|values| values.first().copied()))
        .unwrap_or(default_extension);
    let uri_identity = Sha256::digest(uri.as_bytes());
    let base = output_identity.map(str::to_owned).unwrap_or_else(|| {
        format!(
            "resource-{uri_identity}",
            uri_identity = hex::encode(uri_identity)
        )
    });
    let content_identity = Sha256::digest(&bytes);
    let stem = resource_stem(&uri).unwrap_or_else(|| "resource".into());
    let name = format!(
        "{base}-{content_identity}-{stem}.{extension}",
        content_identity = hex::encode(content_identity)
    );
    let path = workspace.join(name);
    publish_bytes(workspace, &path, &bytes, "resource").await?;
    Ok(path)
}

fn resource_stem(uri: &str) -> Option<String> {
    let parsed = tauri::Url::parse(uri).ok()?;
    // Segments work for every scheme: to_file_path() rejects the drive-less file URLs Windows
    // tests and agents use. Unlike to_file_path(), segments stay percent-encoded, so decode.
    let segment = parsed.path_segments()?.next_back()?;
    let name = percent_encoding::percent_decode_str(segment)
        .decode_utf8()
        .ok()?
        .into_owned();
    let stem = Path::new(&name).file_stem()?.to_str()?;
    if stem.is_empty()
        || stem.starts_with('.')
        || !stem.chars().all(|character| {
            character.is_alphanumeric() || matches!(character, ' ' | '-' | '_' | '.')
        })
    {
        return None;
    }
    // Leave room for the occurrence identity, content digest and extension within a 255-byte
    // filesystem component. Truncate the display stem only, never the resource identity.
    Some(
        stem.chars()
            .scan(0, |bytes, character| {
                *bytes += character.len_utf8();
                (*bytes <= 64).then_some(character)
            })
            .collect(),
    )
}

async fn persist_media(
    workspace: &Path,
    encoded: String,
    mime_type: &str,
    stem: &str,
    identity: Option<&str>,
) -> Result<PathBuf, Error> {
    let parsed_mime = mime_type.parse::<mime::Mime>().ok();
    let maximum_bytes = match parsed_mime.as_ref().map(mime::Mime::type_) {
        Some(mime::IMAGE) => MAX_EMBEDDED_IMAGE_BYTES,
        Some(mime::AUDIO) => MAX_EMBEDDED_AUDIO_BYTES,
        _ => MAX_EMBEDDED_RESOURCE_BYTES,
    };
    let bytes = decode_bounded(encoded, maximum_bytes, "media")?;
    let extension = mime_guess::get_mime_extensions_str(mime_type)
        .and_then(|values| values.first())
        .ok_or_else(|| {
            Error::internal_error().data(format!("Agent returned unsupported media: {mime_type}"))
        })?;
    let occurrence = identity
        .map(str::to_owned)
        .unwrap_or_else(|| format!("media-{}", uuid::Uuid::new_v4()));
    let content_identity = Sha256::digest(&bytes);
    let path = workspace.join(format!(
        "{occurrence}-{content_identity}-{stem}.{extension}",
        content_identity = hex::encode(content_identity)
    ));
    publish_bytes(workspace, &path, &bytes, "media").await?;
    Ok(path)
}

fn decode_bounded(encoded: String, maximum_bytes: usize, label: &str) -> Result<Vec<u8>, Error> {
    if encoded.len() > maximum_bytes.div_ceil(3).saturating_mul(4) {
        return Err(size_limit_error(label));
    }
    let bytes = STANDARD.decode(encoded).map_err(|error| {
        Error::invalid_request().data(format!("Agent returned invalid {label}: {error}"))
    })?;
    bounded_bytes(bytes, maximum_bytes, label)
}

fn bounded_bytes(bytes: Vec<u8>, maximum_bytes: usize, label: &str) -> Result<Vec<u8>, Error> {
    if bytes.len() > maximum_bytes {
        return Err(size_limit_error(label));
    }
    Ok(bytes)
}

fn size_limit_error(label: &str) -> Error {
    Error::invalid_request().data(format!("Agent {label} exceeded its size limit"))
}

async fn publish_bytes(
    workspace: &Path,
    target: &Path,
    bytes: &[u8],
    label: &str,
) -> Result<(), Error> {
    let pending = workspace.join(format!(".{label}-{}.partial", uuid::Uuid::new_v4()));
    let publication = async {
        let mut file = tokio::fs::File::create(&pending).await?;
        file.write_all(bytes).await?;
        file.sync_all().await?;
        drop(file);
        tokio::fs::rename(&pending, target).await
    }
    .await;
    if let Err(error) = publication {
        let _ = tokio::fs::remove_file(&pending).await;
        return Err(Error::internal_error().data(format!("Cannot save Agent {label}: {error}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
