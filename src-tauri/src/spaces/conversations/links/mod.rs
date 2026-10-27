//! A link in a conversation names a file the way its writer saw it. Following it opens that file in
//! its system application — never in the window — and only a file Knovum can vouch for:
//!
//! - a deliverable the Agent wrote in its working directory: publication emptied the directory, so
//!   the path is followed to the canvas node the file became. A node that holds a file opens it; a
//!   Knovum document, which has no file of its own, opens the way the canvas opens it;
//! - a file the Space manages, such as an attachment's source;
//! - a file the user handed Knovum.
//!
//! Any other path is one Knovum has no grant to open on an Agent's word.

use std::path::{Component, Path, PathBuf};

use serde::Serialize;
use sqlx::SqlitePool;
use ts_rs::TS;

use crate::{foundation::app_error::AppError, spaces::db};

#[derive(Debug, PartialEq, Eq, Serialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum ConversationLink {
    // A canvas node that holds no file of its own, for the renderer to open as the canvas does.
    Node { node_id: String },
    // The file, opened in its system application.
    Opened,
}

struct LinkingMessage {
    space_id: String,
    conversation_id: String,
    branch_id: String,
    created_at: i64,
    rowid: i64,
}

pub(crate) async fn follow(
    app: &tauri::AppHandle,
    message_id: &str,
    href: &str,
) -> Result<ConversationLink, AppError> {
    let pool = db::pool(app).await?;
    let message = linking_message(pool, message_id).await?;
    let workspace = crate::spaces::agent_artifacts::workspace::session_directory(
        app,
        &message.space_id,
        &message.conversation_id,
    )?;
    let path = destination(href, &workspace).ok_or_else(unavailable)?;
    let file = match path.strip_prefix(&workspace) {
        Ok(relative) => match deliverable(pool, &message, relative).await? {
            Deliverable::File(asset) => crate::spaces::assets::authorize_spaces_asset(app, &asset)?,
            Deliverable::Document(node_id) => return Ok(ConversationLink::Node { node_id }),
        },
        Err(_) => held_file(app, &path).ok_or_else(unavailable)?,
    };
    crate::spaces::assets::files::open_in_system(app, &file)?;
    Ok(ConversationLink::Opened)
}

/// A file outside the working directory that Knovum holds: the Space's own, or the user's.
fn held_file(app: &tauri::AppHandle, path: &Path) -> Option<PathBuf> {
    crate::spaces::assets::authorize_spaces_asset(app, path)
        .ok()
        .or_else(|| super::upload::authorize(app, path.to_str()?).ok())
}

fn unavailable() -> AppError {
    AppError::new(
        "conversation_link_unavailable",
        "conversation-link-unavailable: the link names no file this Space holds",
    )
}

async fn linking_message(pool: &SqlitePool, message_id: &str) -> Result<LinkingMessage, AppError> {
    let row: Option<(String, String, String, i64, i64)> = sqlx::query_as(
        "SELECT c.space_id,m.conversation_id,m.branch_id,m.created_at,m.rowid
         FROM spaces_conversation_messages m JOIN spaces_conversations c ON c.id=m.conversation_id
         WHERE m.id=$1",
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    let (space_id, conversation_id, branch_id, created_at, rowid) = row.ok_or_else(unavailable)?;
    Ok(LinkingMessage {
        space_id,
        conversation_id,
        branch_id,
        created_at,
        rowid,
    })
}

/// The file a link's destination names. A destination arrives URL-encoded, may carry a fragment
/// or query, and may be relative to the writer's working directory; `..` never climbs above the
/// filesystem root.
fn destination(href: &str, workspace: &Path) -> Option<PathBuf> {
    let encoded = href.split(['#', '?']).next()?;
    let decoded = percent_encoding::percent_decode_str(encoded)
        .decode_utf8()
        .ok()?;
    if decoded.is_empty() {
        return None;
    }
    let named = Path::new(decoded.as_ref());
    let mut path = PathBuf::new();
    for component in workspace.join(named).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !path.pop() {
                    return None;
                }
            }
            other => path.push(other),
        }
    }
    Some(path)
}

#[derive(Debug, PartialEq, Eq)]
enum Deliverable {
    /// The managed file the node holds.
    File(PathBuf),
    /// A Knovum document, by node id.
    Document(String),
}

/// What a deliverable became: the node first published under this path at or after the message
/// that links it, on its branch — the turn that wrote it.
async fn deliverable(
    pool: &SqlitePool,
    message: &LinkingMessage,
    relative: &Path,
) -> Result<Deliverable, AppError> {
    let relative = relative.to_str().ok_or_else(unavailable)?;
    let node: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT n.id,json_extract(n.data,'$.assetPath')
         FROM spaces_conversation_messages m, json_each(m.content,'$.outputs') o
         JOIN spaces_nodes n ON n.id=json_extract(o.value,'$.nodeId')
         WHERE m.conversation_id=$1 AND m.branch_id=$2 AND (m.created_at,m.rowid)>=($3,$4)
           AND json_extract(o.value,'$.workspacePath')=$5
           AND n.space_id=$6 AND n.deleted_at IS NULL
         ORDER BY m.created_at,m.rowid LIMIT 1",
    )
    .bind(&message.conversation_id)
    .bind(&message.branch_id)
    .bind(message.created_at)
    .bind(message.rowid)
    .bind(relative)
    .bind(&message.space_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    match node.ok_or_else(unavailable)? {
        (_, Some(asset)) => Ok(Deliverable::File(PathBuf::from(asset))),
        (node_id, None) => Ok(Deliverable::Document(node_id)),
    }
}

#[cfg(test)]
mod tests;
