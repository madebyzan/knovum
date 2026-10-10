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
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(&app).await?;
    let connector = super::super::connectors::definitions::load(&pool, &connector_id).await?;
    if connector.plugin_id.is_some() {
        return Err("plugin-connections-owned-by-package".into());
    }
    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    let connection = super::connections::create_on(&mut tx, connector_id, name).await?;
    tx.commit().await.map_err(|error| error.to_string())?;
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
    if connection.connector_id != old.connector_id {
        return Err("mcp-connection-service-binding-is-immutable".into());
    }
    let connector =
        super::super::connectors::definitions::load(&pool, &connection.connector_id).await?;
    if connector.plugin_id.is_some() && connection.enabled != old.enabled {
        return Err("plugin-activation-owned-by-package".into());
    }
    connection.apply_binding_change(&old);
    if connector.plugin_id.is_some() {
        connection.enabled = old.enabled;
    }
    let standalone_enabled = connection.enabled && connector.plugin_id.is_none();
    if standalone_enabled {
        connector.configured_endpoint()?;
    }
    if let Some(id) = &connection.account_id {
        if standalone_enabled || old.account_id != connection.account_id {
            let account = super::super::connectors::accounts::load(&pool, id).await?;
            super::super::connectors::accounts::validate_binding(&connector, &account)?;
        }
    } else if standalone_enabled
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
        .bind(sqlx::types::Json(&connection))
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
    super::super::plugins::reconcile_bundled_plugins(&app).await?;
    super::connections::list(&database::pool(&app).await?).await
}

#[tauri::command]
pub async fn check_mcp_connection(
    app: AppHandle,
    connection_id: String,
) -> Result<McpConnectionCatalog, String> {
    let pool = database::pool(&app).await?;
    let connection = super::connections::load(&pool, &connection_id).await?;
    let connector =
        super::super::connectors::definitions::load(&pool, &connection.connector_id).await?;
    super::sessions::with_inspection(&app, &connection, &connector, |session| async move {
        super::catalog::read(&session.service).await
    })
    .await
}

#[tauri::command]
pub async fn delete_mcp_connection(
    app: AppHandle,
    connection_id: String,
    revision: String,
) -> Result<(), String> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(&app).await?;
    let connection = super::connections::load(&pool, &connection_id).await?;
    let connector =
        super::super::connectors::definitions::load(&pool, &connection.connector_id).await?;
    if connector.plugin_id.is_some() {
        return Err("plugin-connections-owned-by-package".into());
    }
    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    super::connections::require_revision_on(&mut tx, &connection_id, &revision).await?;
    sqlx::query("DELETE FROM connections WHERE id=$1")
        .bind(&connection_id)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    tx.commit().await.map_err(|error| error.to_string())?;
    super::sessions::revoke_connection(&app, &connection_id).await;
    Ok(())
}
