//! Where Agents work on a Space conversation's behalf. A conversation owns one directory, and each
//! kind of Agent works in its own directory inside it, because the two keep files differently: a
//! desktop Agent's directory is an outbox that publication drains every turn, while the built-in
//! Agent keeps its files across the conversation's turns. Sharing one directory would let each
//! empty or publish the other's work.
//!
//! The conversation directory is the unit of ownership: it goes when the conversation is deleted,
//! and the startup sweep drops any a conversation no longer claims.

use std::path::PathBuf;

use tauri::{AppHandle, Manager, Runtime};

use crate::foundation::app_error::{AppError, AppResult};

pub(crate) fn root<R: Runtime>(app: &AppHandle<R>) -> AppResult<PathBuf> {
    app.path()
        .app_data_dir()
        .map(|root| root.join("agent-workspaces").join("spaces"))
        .map_err(|error| AppError::internal(error.to_string()))
}

pub(crate) fn space_directory<R: Runtime>(
    app: &AppHandle<R>,
    space_id: &str,
) -> AppResult<PathBuf> {
    let space = uuid::Uuid::parse_str(space_id)
        .map_err(|_| AppError::invalid("Agent session requires a valid Space identifier"))?;
    Ok(root(app)?.join(space.to_string()))
}

pub(crate) fn conversation_directory<R: Runtime>(
    app: &AppHandle<R>,
    space_id: &str,
    conversation_id: &str,
) -> AppResult<PathBuf> {
    let conversation = uuid::Uuid::parse_str(conversation_id)
        .map_err(|_| AppError::invalid("Agent session requires a valid conversation identifier"))?;
    Ok(space_directory(app, space_id)?.join(conversation.to_string()))
}

/// The working directory of a desktop Agent's runtime.
pub(crate) fn desktop_agent_directory<R: Runtime>(
    app: &AppHandle<R>,
    space_id: &str,
    conversation_id: &str,
) -> AppResult<PathBuf> {
    Ok(conversation_directory(app, space_id, conversation_id)?.join("desktop-agent"))
}

/// The built-in Agent's working directory: the root of its file tools and where its commands run.
pub(crate) fn builtin_agent_directory<R: Runtime>(
    app: &AppHandle<R>,
    space_id: &str,
    conversation_id: &str,
) -> AppResult<PathBuf> {
    Ok(conversation_directory(app, space_id, conversation_id)?.join("builtin-agent"))
}
