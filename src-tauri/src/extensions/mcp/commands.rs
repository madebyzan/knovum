use super::super::{
    contracts::{ConnectorAuthorization, ConnectorDefinition, McpConnection, McpConnectionCatalog},
    database, ExtensionState,
};
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn create_mcp_connection(
    app: AppHandle,
    mut connection: McpConnection,
    authorization: ConnectorAuthorization,
) -> Result<McpConnection, String> {
    if connection.name.trim().is_empty() {
        return Err("mcp-connection-name-required".into());
    }
    super::super::connectors::requirements::validate(&authorization)?;
    super::endpoint::validate(&connection.endpoint)?;
    connection.id = uuid::Uuid::new_v4().to_string();
    connection.connector_id = uuid::Uuid::new_v4().to_string();
    connection.revision = uuid::Uuid::new_v4().to_string();
    connection.plugin_id = None;
    connection.component_name = None;
    connection.account_id = None;
    connection.tools.clear();
    let connector = ConnectorDefinition {
        id: connection.connector_id.clone(),
        name: connection.name.clone(),
        plugin_id: None,
        authorization,
    };
    let pool = database::pool(&app).await?;
    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    sqlx::query("INSERT INTO connectors(id,document) VALUES($1,$2)")
        .bind(&connector.id)
        .bind(database::encode(&connector)?)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    sqlx::query("INSERT INTO connections(id,document) VALUES($1,$2)")
        .bind(&connection.id)
        .bind(database::encode(&connection)?)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    tx.commit().await.map_err(|error| error.to_string())?;
    Ok(connection)
}

#[tauri::command]
pub async fn update_mcp_connection(
    app: AppHandle,
    mut connection: McpConnection,
    authorization: Option<ConnectorAuthorization>,
) -> Result<McpConnection, String> {
    if connection.name.trim().is_empty() {
        return Err("mcp-connection-name-required".into());
    }
    super::endpoint::validate(&connection.endpoint)?;
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(&app).await?;
    let old = super::connections::load(&pool, &connection.id).await?;
    if old.revision != connection.revision {
        return Err("mcp-connection-changed-refresh-before-saving".into());
    }
    connection.connector_id = old.connector_id;
    connection.plugin_id = old.plugin_id;
    connection.component_name = old.component_name;
    if old.endpoint != connection.endpoint {
        connection.account_id = None;
        connection.tools.clear();
    }
    if old.account_id != connection.account_id {
        connection.tools.clear();
    }
    if let Some(id) = &connection.account_id {
        let account = super::super::connectors::accounts::load(&pool, id).await?;
        if account.connector_id != connection.connector_id
            || account.resource != super::endpoint::resource(&connection.endpoint)
        {
            return Err("connector-account-binding-mismatch".into());
        }
    }
    let mut connector = super::super::connectors::accounts::definitions(&pool)
        .await?
        .into_iter()
        .find(|connector| connector.id == connection.connector_id)
        .ok_or("connector-definition-missing")?;
    let mut revoked = Vec::new();
    if let Some(authorization) = authorization {
        super::super::connectors::requirements::validate(&authorization)?;
        if authorization != connector.authorization {
            connector.authorization = authorization;
            connection.account_id = None;
            connection.tools.clear();
            revoked = super::super::connectors::accounts::list(&pool)
                .await?
                .into_iter()
                .filter(|account| account.connector_id == connector.id)
                .collect();
        }
    }
    let expected_revision = connection.revision.clone();
    connection.revision = uuid::Uuid::new_v4().to_string();
    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    super::connections::require_revision_on(&mut tx, &connection.id, &expected_revision).await?;
    sqlx::query("UPDATE connectors SET document=$1 WHERE id=$2")
        .bind(database::encode(&connector)?)
        .bind(&connector.id)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    sqlx::query("UPDATE connections SET document=$1 WHERE id=$2")
        .bind(database::encode(&connection)?)
        .bind(&connection.id)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    for account in &mut revoked {
        account.enabled = false;
        account.cleanup_pending = true;
        account.authorization = connector.authorization.clone();
        account.issuer = None;
        account.scopes.clear();
        sqlx::query("UPDATE connector_accounts SET document=$1 WHERE id=$2")
            .bind(database::encode(account)?)
            .bind(&account.id)
            .execute(&mut *tx)
            .await
            .map_err(|error| error.to_string())?;
    }
    tx.commit().await.map_err(|error| error.to_string())?;
    for account in revoked {
        super::super::connectors::commands::disconnect_connector_account(app.clone(), account.id)
            .await?;
    }
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
    let connection = super::connections::load(&database::pool(&app).await?, &id).await?;
    let session = super::client::connect(&app, &connection, None).await?;
    let result = super::catalog::read(&session.service).await;
    session.close().await;
    let current = super::connections::load(&database::pool(&app).await?, &id).await?;
    if current.revision != connection.revision {
        return Err("mcp-connection-changed-during-check".into());
    }
    result
}

#[tauri::command]
pub async fn delete_mcp_connection(app: AppHandle, id: String) -> Result<(), String> {
    let pool = database::pool(&app).await?;
    if super::connections::load(&pool, &id)
        .await?
        .plugin_id
        .is_some()
    {
        return Err("plugin-connection-owned-by-plugin-disable-it-instead".into());
    }
    sqlx::query("DELETE FROM connections WHERE id=$1")
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|error| error.to_string())?;
    super::sessions::revoke_connection(&app, &id).await;
    super::super::connectors::cleanup::collect(&app).await?;
    Ok(())
}
