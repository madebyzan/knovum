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

/// Connects a service, or reconnects one whose authorization is being renewed.
///
/// A service is connected once, so this is the only way an authorization comes into being: the
/// account it writes belongs to the connector and is created on first connect. Reconnecting
/// renews that same account rather than adding a second one.
#[tauri::command]
pub async fn connect_connector(
    app: AppHandle,
    connector_id: String,
    secret_fields: BTreeMap<String, String>,
) -> AppResult<ConnectorAccount> {
    let lock = authorization::account_lock(&app, &connector_id).await;
    let _guard = lock.lock().await;
    let pool = database::pool(&app).await?;
    let connector = super::definitions::load(&pool, &connector_id).await?;
    let mut account = match accounts::of_connector(&pool, &connector_id).await? {
        Some(account) if account.cleanup_pending => {
            return Err(AppError::invalid(
                "connector-account-disconnect-in-progress",
            ))
        }
        Some(account) => account,
        None => {
            let account = ConnectorAccount {
                connector_id: connector.id.clone(),
                revision: uuid::Uuid::new_v4().to_string(),
                resource: super::super::mcp::endpoint::resource(connector.configured_endpoint()?),
                issuer: None,
                scopes: Vec::new(),
                authorization: connector.authorization.clone(),
                enabled: false,
                cleanup_pending: false,
            };
            accounts::save(&pool, &account).await?;
            account
        }
    };
    let fields =
        super::authorization_admission::admit(&pool, &account, &connector, secret_fields).await?;
    super::super::mcp::sessions::revoke_account(&app, &connector_id).await;
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
                crate::secrets::write_secret(&key(&connector_id), &database::encode(&fields)?)
                    .await?;
            }
            ConnectorAuthorization::None => {}
        }
        Ok::<_, String>(())
    }
    .await;
    if let Err(error) = authorization {
        accounts::disable_account(&pool, &connector_id).await?;
        accounts::mark_cleanup_pending(&pool, &connector_id).await?;
        super::credential_store::remove(&app, &connector_id).await?;
        accounts::clear_cleanup_pending(&pool, &connector_id).await?;
        return Err(error.into());
    }
    let current = accounts::load(&pool, &connector_id).await?;
    if current.cleanup_pending
        || current.resource != account.resource
        || current.authorization != account.authorization
    {
        super::credential_store::remove(&app, &connector_id).await?;
        return Err(AppError::invalid(
            "connector-account-disconnected-during-authorization",
        ));
    }
    account.enabled = true;
    account.cleanup_pending = false;
    account.revision = uuid::Uuid::new_v4().to_string();
    if let Err(error) = accounts::commit_authorization(&pool, &account, &current).await {
        super::credential_store::remove(&app, &connector_id).await?;
        return Err(error.into());
    }
    Ok(account)
}

#[tauri::command]
pub async fn cancel_connector_authorization(app: AppHandle, connector_id: String) -> AppResult<()> {
    if let Some(cancellation) = app
        .state::<ExtensionState>()
        .authorizations
        .lock()
        .await
        .get(&connector_id)
        .map(|(_, cancellation)| cancellation.clone())
    {
        cancellation.cancel();
    }
    Ok(())
}

/// Disconnects a service: the authorization stops, sessions are revoked and the credentials go.
#[tauri::command]
pub async fn disconnect_connector(app: AppHandle, connector_id: String) -> AppResult<()> {
    cancel_connector_authorization(app.clone(), connector_id.clone()).await?;
    let pool = database::pool(&app).await?;
    accounts::disable_account(&pool, &connector_id).await?;
    accounts::mark_cleanup_pending(&pool, &connector_id).await?;
    super::super::mcp::sessions::revoke_account(&app, &connector_id).await;
    let lock = authorization::account_lock(&app, &connector_id).await;
    let _guard = lock.lock().await;
    super::credential_store::remove(&app, &connector_id).await?;
    accounts::clear_cleanup_pending(&pool, &connector_id)
        .await
        .map_err(AppError::from)
}
