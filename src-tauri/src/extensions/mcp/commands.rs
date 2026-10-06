use super::super::{
    contracts::{McpConnection, McpConnectionCatalog},
    database, ExtensionState,
};
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn create_mcp_connection(
    app: AppHandle,
    connector_id: String,
    name: String,
) -> Result<McpConnection, String> {
    if name.trim().is_empty() {
        return Err("mcp-connection-name-required".into());
    }
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(&app).await?;
    super::super::connectors::definitions::load(&pool, &connector_id).await?;
    let connection = McpConnection {
        id: uuid::Uuid::new_v4().to_string(),
        connector_id,
        name,
        revision: uuid::Uuid::new_v4().to_string(),
        account_id: None,
        enabled: false,
        tools: Vec::new(),
    };
    sqlx::query("INSERT INTO connections(id,document) VALUES($1,$2)")
        .bind(&connection.id)
        .bind(database::encode(&connection)?)
        .execute(&pool)
        .await
        .map_err(|error| error.to_string())?;
    Ok(connection)
}

#[tauri::command]
pub async fn update_mcp_connection(
    app: AppHandle,
    mut connection: McpConnection,
) -> Result<McpConnection, String> {
    if connection.name.trim().is_empty() {
        return Err("mcp-connection-name-required".into());
    }
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(&app).await?;
    let old = super::connections::load(&pool, &connection.id).await?;
    connection.connector_id = old.connector_id;
    let connector =
        super::super::connectors::definitions::load(&pool, &connection.connector_id).await?;
    if old.account_id != connection.account_id {
        connection.tools.clear();
    }
    if let Some(id) = &connection.account_id {
        if connection.enabled || old.account_id != connection.account_id {
            let account = super::super::connectors::accounts::load(&pool, id).await?;
            super::super::connectors::accounts::validate_binding(&connector, &account)?;
        }
    } else if connection.enabled
        && !matches!(
            connector.authorization,
            super::super::contracts::ConnectorAuthorization::None
        )
    {
        return Err("connector-authorized-account-required".into());
    }
    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    super::connections::require_revision_on(&mut tx, &connection.id, &connection.revision).await?;
    connection.revision = uuid::Uuid::new_v4().to_string();
    sqlx::query("UPDATE connections SET document=$1 WHERE id=$2")
        .bind(database::encode(&connection)?)
        .bind(&connection.id)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    tx.commit().await.map_err(|error| error.to_string())?;
    super::sessions::revoke_connection(&app, &connection.id).await;
    Ok(connection)
}

#[tauri::command]
pub async fn list_mcp_connections(app: AppHandle) -> Result<Vec<McpConnection>, String> {
    super::super::plugins::ensure_builtin(&app).await?;
    super::connections::list(&database::pool(&app).await?).await
}

#[tauri::command]
pub async fn check_mcp_connection(
    app: AppHandle,
    id: String,
) -> Result<McpConnectionCatalog, String> {
    let pool = database::pool(&app).await?;
    let connection = super::connections::load(&pool, &id).await?;
    let connector =
        super::super::connectors::definitions::load(&pool, &connection.connector_id).await?;
    let session = super::client::connect(&app, &connection, &connector, None).await?;
    let result = super::catalog::read(&session.service).await;
    session.close().await;
    let current = super::connections::load(&pool, &id).await?;
    if current.revision != connection.revision {
        return Err("mcp-connection-changed-during-check".into());
    }
    super::super::connectors::definitions::require_current(&pool, &connector).await?;
    result
}

#[tauri::command]
pub async fn delete_mcp_connection(
    app: AppHandle,
    id: String,
    revision: String,
) -> Result<(), String> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(&app).await?;
    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    super::connections::require_revision_on(&mut tx, &id, &revision).await?;
    sqlx::query("DELETE FROM connections WHERE id=$1")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    tx.commit().await.map_err(|error| error.to_string())?;
    super::sessions::revoke_connection(&app, &id).await;
    Ok(())
}
