use super::{admit, prepare};
use crate::extensions::{connectors::accounts, contracts::*};
use sqlx::sqlite::SqlitePoolOptions;
use std::collections::BTreeMap;

fn http() -> McpEndpoint {
    McpEndpoint::StreamableHttp {
        url: "https://example.com/mcp".into(),
        headers: Default::default(),
        allow_local_network: false,
    }
}

#[tokio::test]
async fn invalid_reauthorization_input_leaves_current_authority_unchanged() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("migrations/extensions")
        .run(&pool)
        .await
        .unwrap();
    let account = ConnectorAccount {
        id: "account".into(),
        revision: "authorized".into(),
        connector_id: "service".into(),
        display_name: "Work".into(),
        resource: "https://example.com/mcp".into(),
        issuer: None,
        scopes: vec![],
        authorization: ConnectorAuthorization::Bearer,
        enabled: true,
        cleanup_pending: false,
    };
    accounts::save(&pool, &account).await.unwrap();
    let connector = ConnectorDefinition {
        id: "service".into(),
        revision: "one".into(),
        name: "Service".into(),
        description: "Reference".into(),
        plugin_id: None,
        component_name: None,
        endpoint: Some(http()),
        authorization: account.authorization.clone(),
    };
    for token in ["", "bad\r\nheader"] {
        assert!(admit(
            &pool,
            &account,
            &connector,
            [("token".into(), token.into())].into_iter().collect()
        )
        .await
        .is_err());
        accounts::require_current(&pool, &account).await.unwrap();
    }
}

#[test]
fn secret_fields_reject_invalid_http_values_and_accept_valid_process_values() {
    let authorization = ConnectorAuthorization::SecretFields {
        names: vec!["credential".into()],
    };
    let fields = [("credential".into(), "first\nsecond".into())]
        .into_iter()
        .collect();
    assert!(prepare(&authorization, &http(), fields).is_err());
    let endpoint = McpEndpoint::Stdio {
        executable: "/installed/server".into(),
        args: vec![],
        cwd: None,
        environment: Default::default(),
    };
    assert!(prepare(
        &authorization,
        &endpoint,
        [("credential".into(), "first\nsecond".into())]
            .into_iter()
            .collect()
    )
    .is_ok());
}

#[test]
fn oauth_and_unauthenticated_accounts_do_not_accept_manual_secrets() {
    for authorization in [
        ConnectorAuthorization::None,
        ConnectorAuthorization::OAuth {
            client_id: None,
            scopes: vec![],
        },
    ] {
        assert!(prepare(
            &authorization,
            &http(),
            BTreeMap::from([("token".into(), "secret".into())])
        )
        .is_err());
    }
}
