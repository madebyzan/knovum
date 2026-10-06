use super::super::{
    contracts::{ConnectorAccount, ConnectorAuthorization},
    database, ExtensionState,
};
use super::{
    accounts,
    credential_store::{key, AccountCredentialStore},
};
use rmcp::transport::auth::{
    AuthorizationManager, AuthorizationRequest, InMemoryStateStore, OAuthState,
};
use std::{sync::Arc, time::Duration};
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

pub(super) async fn account_lock(app: &AppHandle, id: &str) -> Arc<tokio::sync::Mutex<()>> {
    app.state::<ExtensionState>()
        .account_locks
        .lock()
        .await
        .entry(id.to_owned())
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone()
}

pub(super) async fn manager(
    app: &AppHandle,
    account: &ConnectorAccount,
    allow_local: bool,
    authorization: Option<tokio_util::sync::CancellationToken>,
) -> Result<AuthorizationManager, String> {
    let mut manager = AuthorizationManager::new_with_oauth_http_client(
        &account.resource,
        super::oauth_http::client(allow_local),
    )
    .await
    .map_err(|_| "connector-oauth-resource-invalid")?;
    manager.set_credential_store(AccountCredentialStore {
        app: app.clone(),
        account_id: account.id.clone(),
        account_revision: account.revision.clone(),
        authorization,
    });
    manager.set_state_store(InMemoryStateStore::new());
    Ok(manager)
}

pub(super) async fn authorize(
    app: &AppHandle,
    account: &mut ConnectorAccount,
    allow_local: bool,
) -> Result<(), String> {
    let cancellation = app.state::<ExtensionState>().shutdown.child_token();
    app.state::<ExtensionState>()
        .authorizations
        .lock()
        .await
        .insert(account.id.clone(), cancellation.clone());
    let result = tokio::select! {
        _ = cancellation.cancelled() => Err("connector-authorization-cancelled".into()),
        result = authorize_flow(app, account, allow_local, cancellation.clone()) => result,
    };
    cancellation.cancel();
    app.state::<ExtensionState>()
        .authorizations
        .lock()
        .await
        .remove(&account.id);
    result
}

async fn authorize_flow(
    app: &AppHandle,
    account: &mut ConnectorAccount,
    allow_local: bool,
    cancellation: tokio_util::sync::CancellationToken,
) -> Result<(), String> {
    let ConnectorAuthorization::OAuth { client_id, scopes } = &account.authorization else {
        return Err("connector-oauth-authorization-required".into());
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| error.to_string())?;
    let callback = format!(
        "http://127.0.0.1:{}/callback",
        listener
            .local_addr()
            .map_err(|error| error.to_string())?
            .port()
    );
    let mut state = OAuthState::Unauthorized(
        manager(app, account, allow_local, Some(cancellation.clone())).await?,
    );
    let mut request = AuthorizationRequest::new(&callback)
        .with_client_name("Knovum")
        .with_application_type("native")
        .with_scopes(scopes.clone());
    if let Some(client_id) = client_id {
        request = request.with_preregistered_client(client_id);
    }
    state.start_authorization(request).await.map_err(|error| {
        format!(
            "connector-oauth-authorization-failed: {}",
            error_category(&error)
        )
    })?;
    let url = state
        .get_authorization_url()
        .await
        .map_err(|_| "connector-oauth-url-unavailable")?;
    let (send, receive) = tokio::sync::oneshot::channel::<String>();
    let sender = Arc::new(std::sync::Mutex::new(Some(send)));
    let router = axum::Router::new().route(
        "/callback",
        axum::routing::get({
            let sender = sender.clone();
            let callback = callback.clone();
            move |uri: axum::http::Uri| {
                let sender = sender.clone();
                let callback = callback.clone();
                async move {
                    let value = format!("{callback}?{}", uri.query().unwrap_or(""));
                    if let Ok(mut sender) = sender.lock() {
                        if let Some(sender) = sender.take() {
                            let _ = sender.send(value);
                        }
                    }
                    "Authorization callback received. You may return to Knovum."
                }
            }
        }),
    );
    let shutdown = cancellation.clone();
    let server = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown.cancelled_owned())
            .await
    });
    let result = async {
        app.opener().open_url(url, None::<String>).map_err(|_| "connector-browser-open-failed")?;
        let callback = tokio::select! {
            _ = cancellation.cancelled() => return Err("connector-authorization-cancelled".into()),
            callback = tokio::time::timeout(Duration::from_secs(180), receive) => callback.map_err(|_| "connector-authorization-timeout")?.map_err(|_| "connector-authorization-callback-closed")?,
        };
        state.handle_callback_url(&callback).await.map_err(|error| format!("connector-oauth-callback-rejected: {}", error_category(&error)))?;
        let stored = crate::secrets::read_secret(&key(&account.id)).await?.ok_or("connector-credentials-not-saved")?;
        let stored: rmcp::transport::auth::StoredCredentials = serde_json::from_str(&stored).map_err(|_| "connector-credentials-invalid")?;
        account.issuer = stored.issuer; account.scopes = stored.granted_scopes;
        Ok(())
    }.await;
    cancellation.cancel();
    let _ = server.await;
    result
}

fn error_category(error: &rmcp::transport::auth::AuthError) -> &'static str {
    use rmcp::transport::auth::AuthError;
    match error {
        AuthError::PkceUnsupported => "provider does not support required PKCE S256",
        AuthError::NoAuthorizationSupport => "provider did not advertise OAuth authorization",
        AuthError::RegistrationFailed(_) => {
            "public client registration was rejected; configure a registered public client"
        }
        AuthError::AuthorizationServerMismatch { .. }
        | AuthError::AuthorizationServerMissingIssuer { .. } => "authorization issuer mismatch",
        AuthError::TokenExchangeFailed(_) => "authorization code exchange was rejected",
        AuthError::HttpError(_) => "authorization endpoint request failed",
        _ => "authorization state or provider metadata was rejected",
    }
}

pub(crate) async fn oauth_client(
    app: &AppHandle,
    account: &ConnectorAccount,
    client: crate::extensions::mcp::http_client::BoundedHttpClient,
    allow_local: bool,
) -> Result<
    rmcp::transport::auth::AuthClient<crate::extensions::mcp::http_client::BoundedHttpClient>,
    String,
> {
    let lock = account_lock(app, &account.id).await;
    let _guard = lock.lock().await;
    let current = accounts::load(&database::pool(app).await?, &account.id).await?;
    if !current.enabled
        || current.cleanup_pending
        || current.resource != account.resource
        || current.revision != account.revision
    {
        return Err("connector-account-revoked-or-resource-changed".into());
    }
    let state = app.state::<ExtensionState>();
    if let Some(client) = state.oauth_clients.lock().await.get(&account.id).cloned() {
        return Ok(client);
    }
    let mut manager = manager(app, account, allow_local, None).await?;
    if !manager
        .initialize_from_store()
        .await
        .map_err(|_| "connector-oauth-stored-identity-rejected")?
    {
        return Err("connector-authorization-required".into());
    }
    let mut clients = state.oauth_clients.lock().await;
    let current = accounts::load(&database::pool(app).await?, &account.id).await?;
    if !current.enabled
        || current.cleanup_pending
        || current.resource != account.resource
        || current.revision != account.revision
    {
        return Err("connector-account-revoked-or-resource-changed".into());
    }
    let client = rmcp::transport::auth::AuthClient::new(client, manager);
    clients.insert(account.id.clone(), client.clone());
    Ok(client)
}
