use std::path::PathBuf;

use lancedb::{connect, Connection, Table};
use tauri::{AppHandle, Manager, Runtime};

use super::schema::TABLE_KNOWLEDGE_CHUNKS;

pub(crate) async fn connect_retrieval_db<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<Connection, String> {
    let path = retrieval_dir(app)?;
    std::fs::create_dir_all(&path)
        .map_err(|error| format!("Failed to create retrieval index directory: {error}"))?;
    connect(path.to_string_lossy().as_ref())
        .manifest_enabled(false)
        .namespace_client_properties([
            ("manifest_enabled", "false"),
            ("dir_listing_enabled", "true"),
            ("dir_listing_to_manifest_migration_enabled", "false"),
        ])
        .execute()
        .await
        .map_err(|error| format!("Failed to open LanceDB retrieval index: {error}"))
}

pub(crate) async fn open_knowledge_table<R: Runtime>(app: &AppHandle<R>) -> Result<Table, String> {
    let db = connect_retrieval_db(app).await?;
    db.open_table(TABLE_KNOWLEDGE_CHUNKS)
        .execute()
        .await
        .map_err(|error| format!("Knowledge retrieval index is not initialized: {error}"))
}

/// Opens the knowledge table if it exists; returns `Ok(None)` when the retrieval index has never
/// been created (fresh install / empty workspace). Any other error — I/O, permission, corruption —
/// is propagated so callers do not silently ignore real problems.
pub(crate) async fn open_knowledge_table_if_exists<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<Option<Table>, String> {
    let db = connect_retrieval_db(app).await?;
    let names = db
        .table_names()
        .execute()
        .await
        .map_err(|error| format!("Failed to inspect LanceDB retrieval tables: {error}"))?;
    if !names.iter().any(|name| name == TABLE_KNOWLEDGE_CHUNKS) {
        return Ok(None);
    }
    db.open_table(TABLE_KNOWLEDGE_CHUNKS)
        .execute()
        .await
        .map(Some)
        .map_err(|error| format!("Failed to open LanceDB knowledge index: {error}"))
}

pub(crate) fn retrieval_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("knowledge-retrieval.lance"))
        .map_err(|error| format!("Failed to resolve app data directory: {error}"))
}
