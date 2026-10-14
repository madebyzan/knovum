use super::super::{
    contracts::{ConnectorDefinition, ExtensionChange, PluginConnectorConfiguration},
    database, ExtensionState,
};
use crate::foundation::app_error::{AppError, AppResult};
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn configure_plugin_connector(
    app: AppHandle,
    connector_id: String,
    revision: String,
    configuration: PluginConnectorConfiguration,
) -> AppResult<ExtensionChange<ConnectorDefinition>> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(&app).await?;
    let previous = super::definitions::load(&pool, &connector_id).await?;
    if previous.revision != revision {
        return Err(AppError::invalid("connector-definition-changed-refresh-before-saving"));
    }
    let plugin_id = previous
        .plugin_id
        .as_ref()
        .ok_or("connector-not-owned-by-plugin")?;
    let plugin = super::super::plugins::load(&pool, plugin_id).await?;
    let declaration = plugin
        .revision
        .connectors
        .iter()
        .find(|declaration| Some(&declaration.component_name) == previous.component_name.as_ref())
        .ok_or("plugin-connector-component-unavailable")?;
    let endpoint = super::super::plugins::process_configuration::configure_endpoint(
        &app,
        &plugin,
        declaration,
        &configuration,
    )?;
    let mut definition = previous.clone();
    definition.endpoint = Some(endpoint);
    definition.revision = uuid::Uuid::new_v4().to_string();
    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    super::definitions::replace_on(&mut tx, &definition, Some(&previous)).await?;
    tx.commit().await.map_err(|error| error.to_string())?;
    super::super::mcp::sessions::revoke_connector(&app, &connector_id).await;
    Ok(ExtensionChange::committed(
        definition,
        [super::cleanup::collect(&app).await],
    ))
}
