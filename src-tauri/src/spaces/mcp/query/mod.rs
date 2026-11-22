mod chunks;
mod contents;
mod graph;
mod history;
mod node;
mod provenance;
mod search;
mod spaces;

use rmcp::ErrorData as McpError;
use serde_json::Value;
use sqlx::{pool::PoolConnection, Sqlite};
use tauri::AppHandle;

use super::support::mcp_error;
use crate::spaces::knowledge::grounding::{number_passages, Reading};

pub(super) use chunks::read_node_chunks;
pub(super) use contents::list_space_contents;
pub(super) use graph::{list_object_types, list_relations};
pub(super) use history::list_knowledge_events;
pub(super) use node::get_node;
pub(super) use provenance::list_provenance;
pub(super) use search::search_knowledge;
pub(super) use spaces::list_spaces;

pub(super) async fn acquire_reader(app: &AppHandle) -> Result<PoolConnection<Sqlite>, McpError> {
    let pool = crate::spaces::db::pool(app).await.map_err(mcp_error)?;
    pool.acquire()
        .await
        .map_err(|error| mcp_error(error.to_string()))
}

/// Acquires a short write transaction solely when `value` contains passages to number, keeping
/// read-heavy node/chunk/search queries off SQLite's write lock.
pub(super) async fn number_if_present(
    app: &AppHandle,
    run_id: &str,
    reading: Reading,
    value: &mut Value,
) -> Result<(), McpError> {
    let has_passages = match reading {
        Reading::Search => value.as_array().is_some_and(|items| !items.is_empty()),
        Reading::Node => value
            .get("chunks")
            .and_then(Value::as_array)
            .is_some_and(|items| !items.is_empty()),
    };
    if !has_passages {
        return Ok(());
    }
    let pool = crate::spaces::db::pool(app).await.map_err(mcp_error)?;
    let mut tx = crate::spaces::db::begin_write(pool)
        .await
        .map_err(mcp_error)?;
    number_passages(&mut tx, run_id, reading, value)
        .await
        .map_err(mcp_error)?;
    tx.commit()
        .await
        .map_err(|error| mcp_error(error.to_string()))
}
