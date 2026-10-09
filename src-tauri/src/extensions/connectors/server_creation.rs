use super::super::{
    contracts::{ConnectorDefinition, ExtensionChange, McpConnection},
    database, ExtensionState,
};
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn create_mcp_server(
    app: AppHandle,
    definition: ConnectorDefinition,
) -> Result<ExtensionChange<McpConnection>, String> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let connection = create(&database::pool(&app).await?, definition).await?;
    Ok(ExtensionChange::committed(connection, []))
}

async fn create(
    pool: &SqlitePool,
    mut definition: ConnectorDefinition,
) -> Result<McpConnection, String> {
    super::definition_commands::validate(&definition)?;
    super::definition_commands::initialize(&mut definition);
    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    super::definitions::replace_on(&mut tx, &definition, None).await?;
    let connection =
        super::super::mcp::connections::create_on(&mut tx, definition.id, definition.name).await?;
    tx.commit().await.map_err(|error| error.to_string())?;
    Ok(connection)
}

#[cfg(test)]
#[path = "tests/server_creation.rs"]
mod tests;
