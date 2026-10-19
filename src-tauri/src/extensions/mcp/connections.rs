use super::super::{contracts::McpConnection, database};
use sqlx::SqlitePool;
use tauri::AppHandle;

pub(in crate::extensions) async fn require_revision_on(
    connection: &mut sqlx::SqliteConnection,
    connection_id: &str,
    revision: &str,
) -> Result<(), String> {
    let current: Option<String> = sqlx::query_scalar(
        "SELECT json_extract(document,'$.revision') FROM connections WHERE id=$1",
    )
    .bind(connection_id)
    .fetch_optional(connection)
    .await
    .map_err(|error| error.to_string())?;
    if current.as_deref() != Some(revision) {
        return Err("mcp-connection-changed-refresh-before-saving".into());
    }
    Ok(())
}

fn decode_connection(
    document: &str,
    definition: Option<&str>,
    owner: Option<&str>,
) -> Result<McpConnection, String> {
    let mut connection: McpConnection = database::decode(document)?;
    let connector: super::super::contracts::ConnectorDefinition =
        database::decode(definition.ok_or("mcp-connector-missing")?)?;
    if connector.plugin_id.is_some() {
        let owner = owner.ok_or("mcp-plugin-missing")?;
        let plugin: super::super::contracts::PluginInstallation = database::decode(owner)?;
        connection.enabled = plugin.enabled;
    }
    Ok(connection)
}

pub(crate) async fn list(pool: &SqlitePool) -> Result<Vec<McpConnection>, String> {
    sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
        "SELECT connections.document, connectors.document, plugins.document FROM connections
         LEFT JOIN connectors ON connectors.id=json_extract(connections.document,'$.connectorId')
         LEFT JOIN plugins ON plugins.id=json_extract(connectors.document,'$.pluginId') ORDER BY connections.id",
    ).fetch_all(pool).await.map_err(|error| error.to_string())?.iter()
        .map(|(document, definition, owner)| decode_connection(document, definition.as_deref(), owner.as_deref())).collect()
}

pub(crate) async fn load(pool: &SqlitePool, connection_id: &str) -> Result<McpConnection, String> {
    let (document, definition, owner): (String, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT connections.document, connectors.document, plugins.document FROM connections
         LEFT JOIN connectors ON connectors.id=json_extract(connections.document,'$.connectorId')
         LEFT JOIN plugins ON plugins.id=json_extract(connectors.document,'$.pluginId') WHERE connections.id=$1",
    ).bind(connection_id).fetch_optional(pool).await.map_err(|error| error.to_string())?
        .ok_or("mcp-connection-not-found")?;
    decode_connection(&document, definition.as_deref(), owner.as_deref())
}

pub(crate) async fn require_current(app: &AppHandle, frozen: &McpConnection) -> Result<(), String> {
    require_access(app, frozen, true).await
}

pub(super) async fn require_reviewable(
    app: &AppHandle,
    frozen: &McpConnection,
) -> Result<(), String> {
    require_access(app, frozen, false).await
}

async fn require_access(
    app: &AppHandle,
    frozen: &McpConnection,
    require_enabled: bool,
) -> Result<(), String> {
    let pool = database::pool(app).await?;
    let current = load(&pool, &frozen.id).await?;
    if (require_enabled && !current.enabled) || current.revision != frozen.revision {
        return Err("mcp-connection-revoked-or-revised".into());
    }
    let connector =
        super::super::connectors::definitions::load(&pool, &current.connector_id).await?;
    connector.configured_endpoint()?;
    if let Some(id) = &connector.plugin_id {
        let plugin = super::super::plugins::load(&pool, id).await?;
        if !plugin.enabled {
            return Err("mcp-plugin-disabled".into());
        }
    }
    match super::super::connectors::accounts::of_connector(&pool, &connector.id).await? {
        Some(account) => {
            super::super::connectors::accounts::validate_binding(&connector, &account)?
        }
        None if matches!(
            connector.authorization,
            super::super::contracts::ConnectorAuthorization::None
        ) => {}
        None => return Err("connector-not-connected".into()),
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/connections.rs"]
mod tests;

pub(in crate::extensions) async fn create_on(
    database: &mut sqlx::SqliteConnection,
    connector_id: String,
    name: String,
) -> Result<McpConnection, String> {
    if name.trim().is_empty() {
        return Err("mcp-connection-name-required".into());
    }
    let connection = McpConnection {
        id: uuid::Uuid::new_v4().to_string(),
        connector_id,
        name,
        revision: uuid::Uuid::new_v4().to_string(),
        enabled: false,
        tools: Vec::new(),
    };
    sqlx::query("INSERT INTO connections(id,document) VALUES($1,$2)")
        .bind(&connection.id)
        .bind(sqlx::types::Json(&connection))
        .execute(database)
        .await
        .map_err(|error| error.to_string())?;
    Ok(connection)
}
