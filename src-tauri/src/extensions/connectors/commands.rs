use super::super::{
    contracts::{ConnectorAccount, ConnectorAuthorization, McpEndpoint},
    database, ExtensionState,
};
use super::{accounts, authorization, credential_store::key};
use crate::foundation::app_error::{AppError, AppResult};
use std::collections::BTreeMap;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn list_connector_accounts(app: AppHandle) -> AppResult<Vec<ConnectorAccount>> {
    accounts::list(&database::pool(&app).await?)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn create_connector_account(
    app: AppHandle,
    connector_id: String,
    display_name: String,
) -> AppResult<ConnectorAccount> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    if display_name.trim().is_empty() {
        return Err(AppError::invalid("connector-account-display-name-required"));
    }
    let pool = database::pool(&app).await?;
    let connector = super::definitions::load(&pool, &connector_id).await?;
    let resource = super::super::mcp::endpoint::resource(connector.configured_endpoint()?);
    let account = ConnectorAccount {
        id: uuid::Uuid::new_v4().to_string(),
        revision: uuid::Uuid::new_v4().to_string(),
        connector_id: connector.id,
        display_name,
        resource,
        issuer: None,
        scopes: Vec::new(),
        authorization: connector.authorization,
        enabled: false,
        cleanup_pending: false,
    };
    accounts::save(&pool, &account).await?;
    Ok(account)
}

#[tauri::command]
pub async fn authorize_connector_account(
    app: AppHandle,
    account_id: String,
    secret_fields: BTreeMap<String, String>,
) -> AppResult<ConnectorAccount> {
    let lock = authorization::account_lock(&app, &account_id).await;
    let _guard = lock.lock().await;
    let pool = database::pool(&app).await?;
    let mut account = accounts::load(&pool, &account_id).await?;
    if account.cleanup_pending {
        return Err(AppError::invalid(
            "connector-account-disconnect-in-progress",
        ));
    }
    let connector = super::definitions::load(&pool, &account.connector_id).await?;
    let fields =
        super::authorization_admission::admit(&pool, &account, &connector, secret_fields).await?;
    super::super::mcp::sessions::revoke_account(&app, &account_id).await;
    let authorization = async {
        match &account.authorization {
            ConnectorAuthorization::OAuth { .. } => {
                let allow_local = matches!(
                    connector.configured_endpoint()?,
                    McpEndpoint::StreamableHttp {
                        allow_local_network: true,
                        ..
                    }
                );
                authorization::authorize(&app, &mut account, allow_local).await?;
            }
            ConnectorAuthorization::Bearer | ConnectorAuthorization::SecretFields { .. } => {
                crate::secrets::write_secret(&key(&account_id), &database::encode(&fields)?)
                    .await?;
            }
            ConnectorAuthorization::None => {}
        }
        Ok::<_, String>(())
    }
    .await;
    if let Err(error) = authorization {
        accounts::set_status(&pool, &account_id, false, true).await?;
        super::credential_store::remove(&app, &account_id).await?;
        accounts::set_status(&pool, &account_id, false, false).await?;
        return Err(error.into());
    }
    let current = accounts::load(&pool, &account_id).await?;
    if current.cleanup_pending
        || current.resource != account.resource
        || current.authorization != account.authorization
    {
        super::credential_store::remove(&app, &account_id).await?;
        return Err(AppError::invalid(
            "connector-account-disconnected-during-authorization",
        ));
    }
    account.enabled = true;
    account.cleanup_pending = false;
    account.revision = uuid::Uuid::new_v4().to_string();
    if let Err(error) = accounts::commit_authorization(&pool, &account, &current).await {
        super::credential_store::remove(&app, &account_id).await?;
        return Err(error.into());
    }
    Ok(account)
}

#[tauri::command]
pub async fn cancel_connector_authorization(app: AppHandle, account_id: String) -> AppResult<()> {
    if let Some(cancellation) = app
        .state::<ExtensionState>()
        .authorizations
        .lock()
        .await
        .get(&account_id)
        .map(|(_, cancellation)| cancellation.clone())
    {
        cancellation.cancel();
    }
    Ok(())
}

#[tauri::command]
pub async fn disconnect_connector_account(app: AppHandle, account_id: String) -> AppResult<()> {
    cancel_connector_authorization(app.clone(), account_id.clone()).await?;
    let pool = database::pool(&app).await?;
    accounts::set_status(&pool, &account_id, false, true).await?;
    super::super::mcp::sessions::revoke_account(&app, &account_id).await;
    let lock = authorization::account_lock(&app, &account_id).await;
    let _guard = lock.lock().await;
    super::credential_store::remove(&app, &account_id).await?;
    accounts::set_status(&pool, &account_id, false, false)
        .await
        .map_err(AppError::from)
}
