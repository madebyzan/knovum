use super::accounts;
use rmcp::transport::auth::{AuthError, CredentialStore, StoredCredentials};

use super::super::{
    contracts::{ConnectorAccount, ConnectorAuthorization},
    database, ExtensionState,
};
use std::{collections::BTreeMap, sync::Arc};
use tauri::{AppHandle, Manager};
pub(super) struct AccountCredentialStore {
    pub app: AppHandle,
    pub account_id: String,
    pub account_revision: String,
    pub authorization: Option<tokio_util::sync::CancellationToken>,
}

fn require_authority(
    account: &ConnectorAccount,
    revision: &str,
    authorization: Option<&tokio_util::sync::CancellationToken>,
) -> Result<(), AuthError> {
    let authorizing = authorization.is_some_and(|token| !token.is_cancelled());
    if account.revision != revision || account.cleanup_pending || (!account.enabled && !authorizing)
    {
        return Err(AuthError::InternalError("connector-account-revoked".into()));
    }
    Ok(())
}

impl AccountCredentialStore {
    async fn require_authority(&self) -> Result<(), AuthError> {
        let account = super::accounts::load(
            &database::pool(&self.app)
                .await
                .map_err(AuthError::InternalError)?,
            &self.account_id,
        )
        .await
        .map_err(AuthError::InternalError)?;
        require_authority(
            &account,
            &self.account_revision,
            self.authorization.as_ref(),
        )
    }
}

pub(super) async fn lock(app: &AppHandle, id: &str) -> Arc<tokio::sync::Mutex<()>> {
    app.state::<ExtensionState>()
        .credential_locks
        .lock()
        .await
        .entry(id.into())
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone()
}

pub(super) async fn remove(app: &AppHandle, id: &str) -> Result<(), String> {
    let lock = lock(app, id).await;
    let _guard = lock.lock().await;
    crate::secrets::remove_secret(&key(id)).await
}

pub(super) fn key(account_id: &str) -> String {
    format!("agent-extension:{account_id}")
}

pub(crate) async fn credentials(
    app: &AppHandle,
    id: &str,
    resource: &str,
) -> Result<BTreeMap<String, String>, String> {
    let lock = super::authorization::account_lock(app, id).await;
    let _guard = lock.lock().await;
    let account = accounts::load(&database::pool(app).await?, id).await?;
    if !account.enabled || account.cleanup_pending || account.resource != resource {
        return Err("connector-account-revoked-or-resource-changed".into());
    }
    match &account.authorization {
        ConnectorAuthorization::None => Ok(BTreeMap::new()),
        ConnectorAuthorization::Bearer | ConnectorAuthorization::SecretFields { .. } => {
            let document = crate::secrets::read_secret(&key(id))
                .await?
                .ok_or("connector-authorization-required")?;
            serde_json::from_str(&document).map_err(|_| "connector-secret-fields-invalid".into())
        }
        ConnectorAuthorization::OAuth { .. } => {
            Err("oauth-credentials-require-sdk-auth-client".into())
        }
    }
}

#[async_trait::async_trait]
impl CredentialStore for AccountCredentialStore {
    async fn load(&self) -> Result<Option<StoredCredentials>, AuthError> {
        let lock = lock(&self.app, &self.account_id).await;
        let _guard = lock.lock().await;
        self.require_authority().await?;
        let secret = crate::secrets::read_secret(&key(&self.account_id))
            .await
            .map_err(AuthError::InternalError)?;
        secret
            .map(|document| {
                serde_json::from_str(&document).map_err(|_| {
                    AuthError::InternalError("stored connector credentials are invalid".into())
                })
            })
            .transpose()
    }
    async fn save(&self, credentials: StoredCredentials) -> Result<(), AuthError> {
        let lock = lock(&self.app, &self.account_id).await;
        let _guard = lock.lock().await;
        self.require_authority().await?;
        let document = serde_json::to_string(&credentials)
            .map_err(|_| AuthError::InternalError("credential serialization failed".into()))?;
        crate::secrets::write_secret(&key(&self.account_id), &document)
            .await
            .map_err(AuthError::InternalError)
    }
    async fn clear(&self) -> Result<(), AuthError> {
        let lock = lock(&self.app, &self.account_id).await;
        let _guard = lock.lock().await;
        self.require_authority().await?;
        crate::secrets::remove_secret(&key(&self.account_id))
            .await
            .map_err(AuthError::InternalError)
    }
}

#[cfg(test)]
#[path = "tests/credentials.rs"]
mod tests;
