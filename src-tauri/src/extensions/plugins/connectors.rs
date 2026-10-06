use super::super::{
    connectors::definitions,
    contracts::{
        ConnectorDefinition, McpConnection, McpEndpoint, PluginConnectorDeclaration,
        PluginInstallation,
    },
    database,
};
use sqlx::SqliteConnection;
use std::collections::BTreeSet;
use tauri::AppHandle;

pub(super) fn resolve(
    app: &AppHandle,
    installation: &PluginInstallation,
) -> Result<Vec<(PluginConnectorDeclaration, McpEndpoint)>, String> {
    installation
        .revision
        .connectors
        .iter()
        .filter(|declaration| declaration.configuration["type"] != "sse")
        .map(|declaration| {
            let endpoint = super::process_configuration::endpoint(
                app,
                &installation.id,
                &installation.revision.digest,
                &declaration.configuration,
            )?;
            super::super::connectors::requirements::validate_access(
                &endpoint,
                &declaration.authorization,
            )?;
            Ok((declaration.clone(), endpoint))
        })
        .collect()
}

pub(super) async fn publish_on(
    tx: &mut SqliteConnection,
    plugin_id: &str,
    revision: &str,
    components: &[(PluginConnectorDeclaration, McpEndpoint)],
) -> Result<Vec<String>, String> {
    let documents: Vec<String> = sqlx::query_scalar(
        "SELECT document FROM connectors WHERE json_extract(document,'$.pluginId')=$1",
    )
    .bind(plugin_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|error| error.to_string())?;
    let previous: Vec<ConnectorDefinition> = documents
        .iter()
        .map(|document| database::decode(document))
        .collect::<Result<_, _>>()?;
    let mut retained = BTreeSet::new();
    let mut revoked = Vec::new();
    for (declaration, endpoint) in components {
        let old = previous.iter().find(|connector| {
            connector.component_name.as_ref() == Some(&declaration.component_name)
        });
        let definition = ConnectorDefinition {
            id: old
                .map(|old| old.id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            revision: revision.into(),
            name: declaration.name.clone(),
            description: declaration.description.clone(),
            plugin_id: Some(plugin_id.into()),
            component_name: Some(declaration.component_name.clone()),
            endpoint: endpoint.clone(),
            authorization: declaration.authorization.clone(),
        };
        revoked.extend(definitions::replace_on(tx, &definition, old).await?);
        if old.is_none() {
            let connection = McpConnection {
                id: uuid::Uuid::new_v4().to_string(),
                connector_id: definition.id.clone(),
                name: definition.name.clone(),
                revision: uuid::Uuid::new_v4().to_string(),
                account_id: None,
                enabled: false,
                tools: Vec::new(),
            };
            sqlx::query("INSERT INTO connections(id,document) VALUES($1,$2)")
                .bind(&connection.id)
                .bind(database::encode(&connection)?)
                .execute(&mut *tx)
                .await
                .map_err(|error| error.to_string())?;
        }
        retained.insert(definition.id);
    }
    for definition in previous
        .iter()
        .filter(|definition| !retained.contains(&definition.id))
    {
        definitions::delete_on(tx, &definition.id).await?;
    }
    Ok(revoked)
}

#[cfg(test)]
#[path = "tests/connectors.rs"]
mod tests;
