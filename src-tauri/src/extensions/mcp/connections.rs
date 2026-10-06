use super::super::{contracts::McpConnection, database};
use sqlx::SqlitePool;
use tauri::AppHandle;

pub(in crate::extensions) async fn require_revision_on(
    connection: &mut sqlx::SqliteConnection,
    id: &str,
    revision: &str,
) -> Result<(), String> {
    let current: Option<String> = sqlx::query_scalar(
        "SELECT json_extract(document,'$.revision') FROM connections WHERE id=$1",
    )
    .bind(id)
    .fetch_optional(connection)
    .await
    .map_err(|error| error.to_string())?;
    if current.as_deref() != Some(revision) {
        return Err("mcp-connection-changed-refresh-before-saving".into());
    }
    Ok(())
}

pub(crate) async fn list(pool: &SqlitePool) -> Result<Vec<McpConnection>, String> {
    sqlx::query_scalar::<_, String>("SELECT document FROM connections ORDER BY id")
        .fetch_all(pool)
        .await
        .map_err(|error| error.to_string())?
        .iter()
        .map(|document| database::decode(document))
        .collect()
}

pub(crate) async fn load(pool: &SqlitePool, id: &str) -> Result<McpConnection, String> {
    let document: String = sqlx::query_scalar("SELECT document FROM connections WHERE id=$1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|error| error.to_string())?
        .ok_or("mcp-connection-not-found")?;
    database::decode(&document)
}

pub(crate) async fn require_current(app: &AppHandle, frozen: &McpConnection) -> Result<(), String> {
    let pool = database::pool(app).await?;
    let current = load(&pool, &frozen.id).await?;
    if !current.enabled
        || current.revision != frozen.revision
        || current.account_id != frozen.account_id
    {
        return Err("mcp-connection-revoked-or-revised".into());
    }
    let connector =
        super::super::connectors::definitions::load(&pool, &current.connector_id).await?;
    if let Some(id) = &connector.plugin_id {
        let plugin = super::super::plugins::load(&pool, id).await?;
        if !plugin.enabled {
            return Err("mcp-plugin-disabled".into());
        }
    }
    if let Some(id) = &current.account_id {
        let account = super::super::connectors::accounts::load(&pool, id).await?;
        super::super::connectors::accounts::validate_binding(&connector, &account)?;
    } else if !matches!(
        connector.authorization,
        super::super::contracts::ConnectorAuthorization::None
    ) {
        return Err("connector-authorized-account-required".into());
    }
    Ok(())
}
