use super::super::{contracts::ConnectorDefinition, database, ExtensionState};
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn list_connector_definitions(
    app: AppHandle,
) -> Result<Vec<ConnectorDefinition>, String> {
    super::super::plugins::ensure_builtin(&app).await?;
    super::definitions::list(&database::pool(&app).await?).await
}

fn validate(definition: &ConnectorDefinition) -> Result<(), String> {
    if definition.name.trim().is_empty() || definition.description.trim().is_empty() {
        return Err("connector-name-and-description-required".into());
    }
    super::requirements::validate_access(&definition.endpoint, &definition.authorization)
}

#[tauri::command]
pub async fn create_connector_definition(
    app: AppHandle,
    mut definition: ConnectorDefinition,
) -> Result<ConnectorDefinition, String> {
    validate(&definition)?;
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    definition.id = uuid::Uuid::new_v4().to_string();
    definition.revision = uuid::Uuid::new_v4().to_string();
    definition.plugin_id = None;
    definition.component_name = None;
    let pool = database::pool(&app).await?;
    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    super::definitions::replace_on(&mut tx, &definition, None).await?;
    tx.commit().await.map_err(|error| error.to_string())?;
    Ok(definition)
}

#[tauri::command]
pub async fn update_connector_definition(
    app: AppHandle,
    mut definition: ConnectorDefinition,
) -> Result<ConnectorDefinition, String> {
    validate(&definition)?;
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(&app).await?;
    let old = super::definitions::load(&pool, &definition.id).await?;
    if old.plugin_id.is_some() {
        return Err("plugin-connector-definition-owned-by-plugin".into());
    }
    if old.revision != definition.revision {
        return Err("connector-definition-changed-refresh-before-saving".into());
    }
    definition.plugin_id = None;
    definition.component_name = None;
    definition.revision = uuid::Uuid::new_v4().to_string();
    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    let revoked = super::definitions::replace_on(&mut tx, &definition, Some(&old)).await?;
    tx.commit().await.map_err(|error| error.to_string())?;
    super::super::mcp::sessions::revoke_connector(&app, &definition.id).await;
    for id in revoked {
        super::commands::disconnect_connector_account(app.clone(), id).await?;
    }
    Ok(definition)
}

#[tauri::command]
pub async fn delete_connector_definition(
    app: AppHandle,
    id: String,
    revision: String,
) -> Result<(), String> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(&app).await?;
    let definition = super::definitions::load(&pool, &id).await?;
    if definition.plugin_id.is_some() {
        return Err("plugin-connector-definition-owned-by-plugin".into());
    }
    if definition.revision != revision {
        return Err("connector-definition-changed-refresh-before-saving".into());
    }
    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    super::definitions::delete_on(&mut tx, &id).await?;
    tx.commit().await.map_err(|error| error.to_string())?;
    super::super::mcp::sessions::revoke_connector(&app, &id).await;
    super::cleanup::collect(&app).await
}
