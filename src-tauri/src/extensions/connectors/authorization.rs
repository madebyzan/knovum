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

/// How long the browser half of an authorization may take. Signing in, approving a workspace and
/// passing a second factor is routinely slower than a couple of minutes, and the clock starts
/// before the browser is even open.
const AUTHORIZATION_DEADLINE: Duration = Duration::from_secs(600);

/// Why the provider turned the authorization down, if it did. OAuth reports a refusal by
/// redirecting back with an `error` parameter rather than a code.
fn denial(query: &str) -> Option<String> {
    let parameters = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect::<Vec<_>>();
    let reported = |name: &str| {
        parameters
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())
    };
    let error = reported("error")?;
    Some(reported("error_description").unwrap_or(error))
}

/// What the browser shows when it comes back. The page states the outcome, tells the user the app
/// has it from here, and closes itself when the browser allows it.
fn callback_page(denial: Option<String>) -> axum::response::Html<String> {
    let (title, detail) = match &denial {
        Some(reason) => ("Authorization was declined", reason.as_str()),
        None => (
            "Authorization complete",
            "Knovum is finishing the connection.",
        ),
    };
    axum::response::Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <title>Knovum</title><style>body{{font:15px -apple-system,BlinkMacSystemFont,\
         'Segoe UI',system-ui,sans-serif;color:#1c1c1e;background:#fff;margin:0;display:flex;\
         min-height:100vh;align-items:center;justify-content:center}}main{{text-align:center;\
         padding:24px}}h1{{font-size:17px;font-weight:600;margin:0 0 8px}}p{{margin:0;color:#6b6b70}}\
         </style></head><body><main><h1>{title}</h1><p>{detail}</p>\
         <p>You can close this tab and return to Knovum.</p></main>\
         <script>setTimeout(function(){{window.close()}},1200)</script></body></html>"
    ))
}

pub(super) async fn account_lock(app: &AppHandle, account_id: &str) -> Arc<tokio::sync::Mutex<()>> {
    app.state::<ExtensionState>()
        .account_locks
        .lock()
        .await
        .entry(account_id.to_owned())
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone()
}

pub(super) async fn create_authorization_manager(
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

/// Runs the browser authorization for an account.
///
/// The token registered here means one thing only: the user asked to stop. The flow never cancels
/// it, so an authorization that finishes is never mistaken for one that was cancelled, and a
/// cancel can only stop the attempt it was asked for.
pub(super) async fn authorize(
    app: &AppHandle,
    account: &mut ConnectorAccount,
    allow_local: bool,
) -> Result<(), String> {
    let attempt = uuid::Uuid::new_v4().to_string();
    let cancellation = app.state::<ExtensionState>().shutdown.child_token();
    supersede(app, &account.id, &attempt, cancellation.clone()).await;
    let authorization = tokio::select! {
        _ = cancellation.cancelled() => Err("connector-authorization-cancelled".into()),
        authorization = authorize_flow(app, account, allow_local, &cancellation) => authorization,
    };
    release(app, &account.id, &attempt).await;
    authorization
}

/// Starts an attempt, stopping whatever attempt that account had in flight: a second browser
/// window for one account is never what the user meant.
async fn supersede(
    app: &AppHandle,
    account_id: &str,
    attempt: &str,
    cancellation: tokio_util::sync::CancellationToken,
) {
    if let Some((_, previous)) = app
        .state::<ExtensionState>()
        .authorizations
        .lock()
        .await
        .insert(account_id.to_owned(), (attempt.to_owned(), cancellation))
    {
        previous.cancel();
    }
}

/// Forgets an attempt, unless a later one has already taken its place.
async fn release(app: &AppHandle, account_id: &str, attempt: &str) {
    let mut authorizations = app.state::<ExtensionState>().authorizations.lock().await;
    if authorizations
        .get(account_id)
        .is_some_and(|(current, _)| current == attempt)
    {
        authorizations.remove(account_id);
    }
}

async fn authorize_flow(
    app: &AppHandle,
    account: &mut ConnectorAccount,
    allow_local: bool,
    cancellation: &tokio_util::sync::CancellationToken,
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
    // The flow's own token stops the callback server and closes the window in which this attempt
    // may write credentials. It is a child of the user's token, so cancelling it when the attempt
    // ends tells nobody that the user cancelled.
    let flow = cancellation.child_token();
    let mut state = OAuthState::Unauthorized(
        create_authorization_manager(app, account, allow_local, Some(flow.clone())).await?,
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
                    let query = uri.query().unwrap_or("").to_owned();
                    let denial = denial(&query);
                    if let Ok(mut sender) = sender.lock() {
                        if let Some(sender) = sender.take() {
                            let _ = sender.send(format!("{callback}?{query}"));
                        }
                    }
                    callback_page(denial)
                }
            }
        }),
    );
    let shutdown = flow.clone();
    let server = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown.cancelled_owned())
            .await
    });
    let authorization = async {
        app.opener().open_url(url, None::<String>).map_err(|_| "connector-browser-open-failed")?;
        let callback = tokio::select! {
            _ = cancellation.cancelled() => return Err("connector-authorization-cancelled".into()),
            callback = tokio::time::timeout(AUTHORIZATION_DEADLINE, receive) => callback.map_err(|_| "connector-authorization-timeout")?.map_err(|_| "connector-authorization-callback-closed")?,
        };
        // A provider that reports a denial has nothing to exchange, and saying so beats reporting
        // a rejected authorization code the provider never issued.
        if let Some(reason) = denial(url::Url::parse(&callback).map_err(|_| "connector-authorization-callback-invalid")?.query().unwrap_or("")) {
            return Err(format!("connector-authorization-denied: {reason}"));
        }
        state.handle_callback_url(&callback).await.map_err(|error| format!("connector-oauth-callback-rejected: {}", error_category(&error)))?;
        let stored = crate::secrets::read_secret(&key(&account.id)).await?.ok_or("connector-credentials-not-saved")?;
        let stored: rmcp::transport::auth::StoredCredentials = serde_json::from_str(&stored).map_err(|_| "connector-credentials-invalid")?;
        account.issuer = stored.issuer; account.scopes = stored.granted_scopes;
        Ok(())
    }.await;
    flow.cancel();
    let _ = server.await;
    authorization
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
    let mut manager = create_authorization_manager(app, account, allow_local, None).await?;
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

#[cfg(test)]
#[path = "tests/authorization.rs"]
mod tests;
