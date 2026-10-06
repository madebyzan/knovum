use super::super::{
    connectors::definitions,
    contracts::{
        ConnectorDefinition, McpConnection, McpEndpoint, PluginConnectorConfiguration,
        PluginConnectorDeclaration, PluginInstallation,
    },
    database,
};
use sqlx::SqliteConnection;
use std::collections::BTreeSet;
use tauri::AppHandle;

pub(super) fn resolve(
    app: &AppHandle,
    installation: &PluginInstallation,
    previous: Option<&PluginInstallation>,
    definitions: &[ConnectorDefinition],
) -> Result<Vec<(PluginConnectorDeclaration, Option<McpEndpoint>)>, String> {
    installation
        .revision
        .connectors
        .iter()
        .map(|declaration| {
            let old_declaration = previous.and_then(|plugin| {
                plugin
                    .revision
                    .connectors
                    .iter()
                    .find(|old| old.component_name == declaration.component_name)
            });
            let old_definition = definitions.iter().find(|definition| {
                definition.plugin_id.as_ref() == Some(&installation.id)
                    && definition.component_name.as_ref() == Some(&declaration.component_name)
            });
            let configuration =
                retained_configuration(declaration, old_declaration, old_definition);
            let (root, data) = super::process_configuration::paths(
                app,
                &installation.id,
                &installation.revision.digest,
            )?;
            let endpoint = super::process_configuration::endpoint(
                &root,
                &data,
                &declaration.configuration,
                configuration.as_ref(),
            )?;
            Ok((declaration.clone(), endpoint))
        })
        .collect()
}

fn retained_configuration(
    declaration: &PluginConnectorDeclaration,
    previous: Option<&PluginConnectorDeclaration>,
    definition: Option<&ConnectorDefinition>,
) -> Option<PluginConnectorConfiguration> {
    let previous = previous?;
    let endpoint = definition?.endpoint.as_ref()?;
    match endpoint {
        McpEndpoint::Stdio { executable, .. }
            if declaration.configuration["type"] == "stdio"
                && previous.configuration["command"] == declaration.configuration["command"] =>
        {
            Some(PluginConnectorConfiguration::Stdio {
                executable: executable.clone(),
            })
        }
        McpEndpoint::StreamableHttp {
            allow_local_network,
            ..
        } if declaration.configuration["type"] == "streamable-http"
            && previous.configuration["url"] == declaration.configuration["url"] =>
        {
            Some(PluginConnectorConfiguration::StreamableHttp {
                allow_local_network: *allow_local_network,
            })
        }
        _ => None,
    }
}

pub(super) async fn publish_on(
    tx: &mut SqliteConnection,
    plugin_id: &str,
    revision: &str,
    components: &[(PluginConnectorDeclaration, Option<McpEndpoint>)],
) -> Result<(), String> {
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
        definitions::replace_on(tx, &definition, old).await?;
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
    Ok(())
}

#[cfg(test)]
#[path = "tests/connectors.rs"]
mod tests;
