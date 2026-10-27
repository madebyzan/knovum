use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use tauri::AppHandle;

use crate::spaces::db;

/// Drops every Agent workspace nothing can reach any more: the workspaces of deleted Spaces and,
/// inside a live Space, every directory that is not one of its conversations. It runs before
/// the Space runtime is ready, so no Agent runtime is working in a workspace the sweep reads.
pub async fn discard_orphaned_workspaces(app: &AppHandle) -> Result<(), String> {
    let root = super::workspace::root(app).map_err(|error| error.to_string())?;
    if !root.try_exists().map_err(|error| error.to_string())? {
        return Ok(());
    }
    sweep(&root, &owners(db::pool(app).await?).await?)
}

/// Live Space ids, each with the conversations allowed to keep a workspace inside it. Both are in
/// the canonical form `workspace::conversation_directory` names directories by.
type Owners = HashMap<String, HashSet<String>>;

async fn owners(pool: &sqlx::SqlitePool) -> Result<Owners, String> {
    let spaces: Vec<String> = sqlx::query_scalar("SELECT id FROM spaces WHERE kind='space'")
        .fetch_all(pool)
        .await
        .map_err(db::sql_error)?;
    let mut owners = spaces
        .iter()
        .map(|id| {
            uuid::Uuid::parse_str(id)
                .map(|space| (space.to_string(), HashSet::new()))
                .map_err(|_| "Space has an invalid identifier".to_owned())
        })
        .collect::<Result<Owners, _>>()?;
    let conversations: Vec<(String, String)> =
        sqlx::query_as("SELECT space_id,id FROM spaces_conversations")
            .fetch_all(pool)
            .await
            .map_err(db::sql_error)?;
    // A conversation whose ids are not UUIDs was never given a workspace, so it owns none.
    for (space, conversation) in conversations {
        if let (Ok(space), Ok(conversation)) = (
            uuid::Uuid::parse_str(&space),
            uuid::Uuid::parse_str(&conversation),
        ) {
            if let Some(sessions) = owners.get_mut(&space.to_string()) {
                sessions.insert(conversation.to_string());
            }
        }
    }
    Ok(owners)
}

fn sweep(root: &Path, owners: &Owners) -> Result<(), String> {
    for entry in std::fs::read_dir(root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let space = entry
            .file_name()
            .into_string()
            .ok()
            .and_then(|name| uuid::Uuid::parse_str(&name).ok())
            .ok_or("Agent workspace has an invalid Space identifier")?;
        match owners.get(&space.to_string()) {
            None => discard(&entry.path())?,
            Some(sessions) if is_directory(&entry)? => sweep_sessions(&entry.path(), sessions)?,
            Some(_) => {}
        }
    }
    Ok(())
}

/// A conversation directory belongs to its Space while the conversation exists; files
/// beside the conversations are not workspaces and are left alone.
fn sweep_sessions(space: &Path, sessions: &HashSet<String>) -> Result<(), String> {
    for entry in std::fs::read_dir(space).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let owned = entry
            .file_name()
            .to_str()
            .is_some_and(|name| sessions.contains(name));
        if !owned && is_directory(&entry)? {
            discard(&entry.path())?;
        }
    }
    Ok(())
}

fn is_directory(entry: &std::fs::DirEntry) -> Result<bool, String> {
    entry
        .file_type()
        .map(|kind| kind.is_dir())
        .map_err(|error| error.to_string())
}

fn discard(path: &Path) -> Result<(), String> {
    crate::external_agents::artifacts::discard(path).map_err(|error| error.to_string())
}

#[cfg(test)]
#[path = "tests/recovery.rs"]
mod tests;
