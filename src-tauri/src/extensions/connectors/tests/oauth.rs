use rmcp::transport::auth::{
    AuthorizationManager, AuthorizationRequest, CredentialStore, InMemoryCredentialStore,
    OAuthState,
};
use serde_json::json;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;

struct OAuthFixture {
    resource: String,
    requests: Arc<Mutex<Vec<BTreeMap<String, String>>>>,
    stop: CancellationToken,
    server: tokio::task::JoinHandle<()>,
}
impl OAuthFixture {
    async fn start() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let resource = format!("{origin}/mcp");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let observed = requests.clone();
        let protected = resource.clone();
        let issuer = origin.clone();
        let oauth = origin.clone();
        let router=axum::Router::new()
            .route("/.well-known/oauth-protected-resource/mcp",axum::routing::get(move|| { let protected=protected.clone();let issuer=issuer.clone(); async move { axum::Json(json!({"resource":protected,"authorization_servers":[issuer],"scopes_supported":["read"]})) } }))
            .route("/.well-known/oauth-authorization-server",axum::routing::get(move|| { let oauth=oauth.clone();async move { axum::Json(json!({"issuer":oauth,"authorization_endpoint":format!("{oauth}/authorize"),"token_endpoint":format!("{oauth}/token"),"response_types_supported":["code"],"grant_types_supported":["authorization_code","refresh_token"],"code_challenge_methods_supported":["S256"],"token_endpoint_auth_methods_supported":["none"],"scopes_supported":["read"]})) } }))
            .route("/token",axum::routing::post(move|body:String| { let observed=observed.clone();async move {
                let parameters=url::form_urlencoded::parse(body.as_bytes()).into_owned().collect::<BTreeMap<_,_>>();let refresh=parameters.get("grant_type").is_some_and(|grant|grant=="refresh_token");observed.lock().unwrap().push(parameters);
                axum::Json(json!({"access_token":if refresh {"refreshed-token"} else {"authorized-token"},"token_type":"Bearer","refresh_token":"refresh-token","expires_in":if refresh {3600} else {0},"scope":"read"}))
            } }));
        let stop = CancellationToken::new();
        let shutdown = stop.clone();
        let server = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(shutdown.cancelled_owned())
                .await
                .unwrap();
        });
        Self {
            resource,
            requests,
            stop,
            server,
        }
    }
    async fn state(&self, store: InMemoryCredentialStore) -> OAuthState {
        let mut manager = AuthorizationManager::new_with_oauth_http_client(
            &self.resource,
            super::oauth_http::client(true),
        )
        .await
        .unwrap();
        manager.set_credential_store(store);
        let mut state = OAuthState::Unauthorized(manager);
        state
            .start_authorization(
                AuthorizationRequest::new("http://127.0.0.1:43210/callback")
                    .with_preregistered_client("fixture-public-client")
                    .with_scopes(["read"]),
            )
            .await
            .unwrap();
        state
    }
    async fn close(self) {
        self.stop.cancel();
        self.server.await.unwrap();
    }
}
#[tokio::test]
async fn sdk_oauth_uses_pkce_state_resource_and_refresh_with_bounded_host_http() {
    let fixture = OAuthFixture::start().await;
    let store = InMemoryCredentialStore::new();
    let mut state = fixture.state(store.clone()).await;
    let url = url::Url::parse(&state.get_authorization_url().await.unwrap()).unwrap();
    let parameters = url.query_pairs().into_owned().collect::<BTreeMap<_, _>>();
    assert_eq!(parameters["code_challenge_method"], "S256");
    assert_eq!(parameters["resource"], fixture.resource);
    assert!(!parameters["state"].is_empty());
    state
        .handle_callback_url(&format!(
            "http://127.0.0.1:43210/callback?code=valid-code&state={}",
            parameters["state"]
        ))
        .await
        .unwrap();
    let mut refresh = AuthorizationManager::new_with_oauth_http_client(
        &fixture.resource,
        super::oauth_http::client(true),
    )
    .await
    .unwrap();
    refresh.set_credential_store(store.clone());
    assert!(refresh.initialize_from_store().await.unwrap());
    assert_eq!(refresh.get_access_token().await.unwrap(), "refreshed-token");
    let stored = store.load().await.unwrap().unwrap();
    assert_eq!(stored.granted_scopes, vec!["read"]);
    {
        let requests = fixture.requests.lock().unwrap();
        assert_eq!(requests.len(), 2);
        assert!(requests[0].contains_key("code_verifier"));
        assert_eq!(requests[0]["resource"], fixture.resource);
        assert_eq!(requests[1]["grant_type"], "refresh_token");
    }
    fixture.close().await;
}
#[tokio::test]
async fn wrong_callback_state_never_exchanges_a_code() {
    let fixture = OAuthFixture::start().await;
    let mut state = fixture.state(InMemoryCredentialStore::new()).await;
    assert!(state
        .handle_callback_url("http://127.0.0.1:43210/callback?code=valid-code&state=wrong")
        .await
        .is_err());
    assert!(fixture.requests.lock().unwrap().is_empty());
    fixture.close().await;
}

#[tokio::test]
async fn accepted_callback_cannot_be_reused_to_exchange_another_code() {
    let fixture = OAuthFixture::start().await;
    let mut state = fixture.state(InMemoryCredentialStore::new()).await;
    let url = url::Url::parse(&state.get_authorization_url().await.unwrap()).unwrap();
    let parameters = url.query_pairs().into_owned().collect::<BTreeMap<_, _>>();
    let callback = format!(
        "http://127.0.0.1:43210/callback?code=valid-code&state={}",
        parameters["state"]
    );
    state.handle_callback_url(&callback).await.unwrap();
    assert!(state.handle_callback_url(&callback).await.is_err());
    assert_eq!(fixture.requests.lock().unwrap().len(), 1);
    fixture.close().await;
}
