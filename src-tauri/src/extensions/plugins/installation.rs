use super::super::{
    content::{self, PackageFiles},
    contracts::{
        ConnectorDefinition, McpConnection, PluginInstallation, PluginPublication, PluginSource,
    },
    database,
};
use super::packages::ReviewedPackage;
use sqlx::SqlitePool;
use tauri::AppHandle;

pub(crate) async fn list(pool: &SqlitePool) -> Result<Vec<PluginInstallation>, String> {
    sqlx::query_scalar::<_, String>("SELECT document FROM plugins ORDER BY id")
        .fetch_all(pool)
        .await
        .map_err(|error| error.to_string())?
        .iter()
        .map(|document| database::decode(document))
        .collect()
}

pub(crate) async fn load(pool: &SqlitePool, id: &str) -> Result<PluginInstallation, String> {
    let document: String = sqlx::query_scalar("SELECT document FROM plugins WHERE id=$1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|error| error.to_string())?
        .ok_or("plugin-not-found")?;
    database::decode(&document)
}

pub(super) async fn commit(
    app: &AppHandle,
    id: &str,
    source: PluginSource,
    reviewed: ReviewedPackage,
    files: PackageFiles,
    builtin: bool,
    publication: Option<PluginPublication>,
) -> Result<PluginInstallation, String> {
    let pool = database::pool(app).await?;
    let previous = list(&pool)
        .await?
        .into_iter()
        .find(|plugin| plugin.id == id);
    let enabled = previous
        .as_ref()
        .map(|plugin| plugin.enabled)
        .unwrap_or(true);
    let installation = PluginInstallation {
        id: id.into(),
        source,
        revision: reviewed.inspection,
        enabled,
        builtin,
        publication,
    };
    let directory = content::root(app)?
        .join("plugins")
        .join(id)
        .join("revisions")
        .join(&installation.revision.digest);
    crate::blocking::run(move || content::publish(&directory, &files)).await?;
    let previous_skills = super::super::skills::list(&pool).await?;
    let previous_connectors = super::super::connectors::accounts::definitions(&pool).await?;
    let previous_connections = super::super::mcp::connections::list(&pool).await?;
    let mut revoked = Vec::new();
    let accounts = super::super::connectors::accounts::list(&pool).await?;
    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    for connection in previous_connections
        .iter()
        .filter(|connection| connection.plugin_id.as_deref() == Some(id))
    {
        super::super::mcp::connections::require_revision_on(
            &mut tx,
            &connection.id,
            &connection.revision,
        )
        .await?;
    }
    sqlx::query("INSERT INTO plugins(id,document) VALUES($1,$2) ON CONFLICT(id) DO UPDATE SET document=excluded.document")
        .bind(id).bind(database::encode(&installation)?).execute(&mut *tx).await.map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM skills WHERE json_extract(document,'$.pluginId')=$1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    for mut skill in reviewed.skills {
        if let Some(previous) = previous_skills.iter().find(|previous| {
            previous.plugin_id.as_deref() == Some(id) && previous.name == skill.name
        }) {
            skill.id = previous.id.clone();
            skill.enabled = previous.enabled;
        }
        skill.plugin_id = Some(id.into());
        sqlx::query("INSERT INTO skills(id,document) VALUES($1,$2)")
            .bind(&skill.id)
            .bind(database::encode(&skill)?)
            .execute(&mut *tx)
            .await
            .map_err(|error| error.to_string())?;
    }
    sqlx::query("DELETE FROM connections WHERE json_extract(document,'$.pluginId')=$1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    for (name, server) in reviewed.mcp_servers {
        if server["type"] == "sse" {
            continue;
        }
        let endpoint = super::process_configuration::endpoint(
            app,
            id,
            &installation.revision.digest,
            &server,
        )?;
        let authorization = reviewed
            .authorizations
            .get(&name)
            .cloned()
            .expect("reviewed connector authorization");
        let old = previous_connections.iter().find(|connection| {
            connection.plugin_id.as_deref() == Some(id)
                && connection.component_name.as_deref() == Some(&name)
        });
        let same_endpoint = old.is_some_and(|old| {
            old.endpoint == endpoint
                && previous_connectors
                    .iter()
                    .find(|connector| connector.id == old.connector_id)
                    .is_some_and(|connector| connector.authorization == authorization)
        });
        let connection = McpConnection {
            id: old
                .map(|old| old.id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            connector_id: old
                .map(|old| old.connector_id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            name: name.clone(),
            plugin_id: Some(id.into()),
            component_name: Some(name),
            revision: installation.revision.digest.clone(),
            endpoint,
            account_id: old
                .filter(|_| same_endpoint)
                .and_then(|old| old.account_id.clone()),
            enabled: old.is_some_and(|old| same_endpoint && old.enabled),
            tools: old
                .filter(|_| same_endpoint)
                .map(|old| old.tools.clone())
                .unwrap_or_default(),
        };
        let authorization_changed = previous_connectors
            .iter()
            .find(|connector| connector.id == connection.connector_id)
            .is_some_and(|connector| connector.authorization != authorization);
        if authorization_changed {
            for mut account in accounts
                .iter()
                .filter(|account| account.connector_id == connection.connector_id)
                .cloned()
            {
                account.enabled = false;
                account.cleanup_pending = true;
                account.authorization = authorization.clone();
                account.issuer = None;
                account.scopes.clear();
                sqlx::query("UPDATE connector_accounts SET document=$1 WHERE id=$2")
                    .bind(database::encode(&account)?)
                    .bind(&account.id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|error| error.to_string())?;
                revoked.push(account.id);
            }
        }
        let connector = ConnectorDefinition {
            id: connection.connector_id.clone(),
            name: reviewed.connector_names[connection
                .component_name
                .as_ref()
                .expect("plugin component")]
            .clone(),
            plugin_id: Some(id.into()),
            authorization,
        };
        sqlx::query("INSERT INTO connectors(id,document) VALUES($1,$2) ON CONFLICT(id) DO UPDATE SET document=excluded.document").bind(&connector.id).bind(database::encode(&connector)?).execute(&mut *tx).await.map_err(|error| error.to_string())?;
        sqlx::query("INSERT INTO connections(id,document) VALUES($1,$2)")
            .bind(&connection.id)
            .bind(database::encode(&connection)?)
            .execute(&mut *tx)
            .await
            .map_err(|error| error.to_string())?;
    }
    tx.commit().await.map_err(|error| error.to_string())?;
    super::super::mcp::sessions::revoke_plugin(app, id).await;
    for account in revoked {
        super::super::connectors::commands::disconnect_connector_account(app.clone(), account)
            .await?;
    }
    super::super::connectors::cleanup::collect(app).await?;
    super::super::reclamation::collect(app).await?;
    Ok(installation)
}
