use crate::extensions::{
    connectors::{accounts, definitions, requirements},
    contracts::*,
    database,
    mcp::connections,
};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

fn definition(id: &str) -> ConnectorDefinition {
    ConnectorDefinition {
        id: id.into(),
        revision: "service-one".into(),
        name: "GitHub".into(),
        description: "Read issues and manage repositories".into(),
        plugin_id: None,
        component_name: None,
        endpoint: McpEndpoint::StreamableHttp {
            url: "https://example.com/mcp".into(),
            headers: Default::default(),
            allow_local_network: false,
        },
        authorization: ConnectorAuthorization::Bearer,
    }
}

async fn fixture() -> (SqlitePool, ConnectorDefinition, ConnectorAccount) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("migrations/extensions")
        .run(&pool)
        .await
        .unwrap();
    let connector = definition("shared");
    let mut tx = pool.begin().await.unwrap();
    for connector in [&connector, &definition("unrelated")] {
        definitions::replace_on(&mut tx, connector, None)
            .await
            .unwrap();
    }
    tx.commit().await.unwrap();
    let account = ConnectorAccount {
        id: "account".into(),
        revision: "authorized-one".into(),
        connector_id: connector.id.clone(),
        display_name: "Work".into(),
        resource: "https://example.com/mcp".into(),
        issuer: None,
        scopes: vec![],
        authorization: ConnectorAuthorization::Bearer,
        enabled: true,
        cleanup_pending: false,
    };
    accounts::save(&pool, &account).await.unwrap();
    let mut other_account = account.clone();
    other_account.id = "other-account".into();
    other_account.connector_id = "unrelated".into();
    accounts::save(&pool, &other_account).await.unwrap();
    for (id, connector_id) in [
        ("personal", "shared"),
        ("work", "shared"),
        ("other", "unrelated"),
    ] {
        let connection = McpConnection {
            id: id.into(),
            connector_id: connector_id.into(),
            name: id.into(),
            revision: "instance-one".into(),
            account_id: Some(if connector_id == "shared" {
                account.id.clone()
            } else {
                other_account.id.clone()
            }),
            enabled: true,
            tools: vec![McpToolPermission {
                name: "list_issues".into(),
                schema_digest: "reviewed".into(),
                read_only: true,
                policy: McpToolPolicy::Allow,
            }],
        };
        sqlx::query("INSERT INTO connections(id,document) VALUES($1,$2)")
            .bind(id)
            .bind(database::encode(&connection).unwrap())
            .execute(&pool)
            .await
            .unwrap();
    }
    (pool, connector, account)
}

#[tokio::test]
async fn service_description_changes_preserve_shared_accounts_and_instance_policies() {
    let (pool, old, account) = fixture().await;
    let mut updated = old.clone();
    updated.description = "Updated service description".into();
    updated.revision = "service-two".into();
    let mut tx = pool.begin().await.unwrap();
    assert!(definitions::replace_on(&mut tx, &updated, Some(&old))
        .await
        .unwrap()
        .is_empty());
    tx.commit().await.unwrap();
    assert!(definitions::require_current(&pool, &old).await.is_err());
    definitions::require_current(&pool, &updated).await.unwrap();
    accounts::require_current(&pool, &account).await.unwrap();
    for id in ["personal", "work"] {
        let connection = connections::load(&pool, id).await.unwrap();
        assert!(connection.enabled);
        assert_eq!(connection.account_id.as_deref(), Some("account"));
        assert_eq!(connection.tools.len(), 1);
        assert_eq!(connection.name, id);
        assert_ne!(connection.revision, "instance-one");
    }
    assert_eq!(
        connections::load(&pool, "other").await.unwrap().revision,
        "instance-one"
    );
}

#[tokio::test]
async fn endpoint_or_authorization_changes_revoke_all_shared_instances_atomically() {
    for change_endpoint in [true, false] {
        let (pool, old, frozen_account) = fixture().await;
        let mut updated = old.clone();
        updated.revision = "service-two".into();
        if change_endpoint {
            updated.endpoint = McpEndpoint::StreamableHttp {
                url: "https://new.example.com/mcp".into(),
                headers: Default::default(),
                allow_local_network: false,
            };
        } else {
            updated.authorization = ConnectorAuthorization::SecretFields {
                names: vec!["X-API-Key".into()],
            };
        }
        let mut tx = pool.begin().await.unwrap();
        assert_eq!(
            definitions::replace_on(&mut tx, &updated, Some(&old))
                .await
                .unwrap(),
            vec!["account"]
        );
        tx.commit().await.unwrap();
        let account = accounts::load(&pool, "account").await.unwrap();
        assert!(!account.enabled);
        assert!(account.cleanup_pending);
        assert_eq!(
            account.resource,
            crate::extensions::mcp::endpoint::resource(&updated.endpoint)
        );
        assert_eq!(account.authorization, updated.authorization);
        assert_ne!(account.revision, frozen_account.revision);
        assert!(accounts::require_current(&pool, &frozen_account)
            .await
            .is_err());
        for id in ["personal", "work"] {
            let connection = connections::load(&pool, id).await.unwrap();
            assert!(!connection.enabled);
            assert!(connection.account_id.is_none());
            assert!(connection.tools.is_empty());
            assert_ne!(connection.revision, "instance-one");
        }
        assert!(connections::load(&pool, "other").await.unwrap().enabled);
    }
}

#[tokio::test]
async fn stale_definition_cannot_change_accounts_or_instances() {
    let (pool, current, account) = fixture().await;
    let mut stale = current.clone();
    stale.revision = "obsolete".into();
    let mut incoming = current.clone();
    incoming.authorization = ConnectorAuthorization::None;
    let mut tx = pool.begin().await.unwrap();
    assert_eq!(
        definitions::replace_on(&mut tx, &incoming, Some(&stale))
            .await
            .unwrap_err(),
        "connector-definition-changed-refresh-before-saving"
    );
    tx.rollback().await.unwrap();
    accounts::require_current(&pool, &account).await.unwrap();
    assert_eq!(
        connections::load(&pool, "work").await.unwrap().revision,
        "instance-one"
    );
    definitions::require_current(&pool, &current).await.unwrap();
}

#[tokio::test]
async fn retiring_service_removes_only_its_instances_and_retains_credentials_for_cleanup() {
    let (pool, old, _) = fixture().await;
    let mut tx = pool.begin().await.unwrap();
    definitions::delete_on(&mut tx, &old.id).await.unwrap();
    tx.commit().await.unwrap();
    assert!(definitions::load(&pool, "shared").await.is_err());
    assert!(connections::load(&pool, "personal").await.is_err());
    assert!(connections::load(&pool, "work").await.is_err());
    assert!(connections::load(&pool, "other").await.is_ok());
    assert!(definitions::load(&pool, "unrelated").await.is_ok());
    assert!(accounts::load(&pool, "account").await.is_ok());
}

#[test]
fn account_authority_is_scoped_to_one_service_definition() {
    let connector = definition("service");
    let mut account = ConnectorAccount {
        id: "account".into(),
        revision: "authorized".into(),
        connector_id: "other".into(),
        display_name: "Work".into(),
        resource: "https://example.com/mcp".into(),
        issuer: None,
        scopes: vec![],
        authorization: ConnectorAuthorization::Bearer,
        enabled: true,
        cleanup_pending: false,
    };
    assert!(accounts::validate_binding(&connector, &account).is_err());
    account.connector_id = connector.id.clone();
    accounts::validate_binding(&connector, &account).unwrap();
    account.authorization = ConnectorAuthorization::None;
    assert!(accounts::validate_binding(&connector, &account).is_err());
}

#[test]
fn http_token_authorization_cannot_be_used_as_stdio_environment() {
    let endpoint = McpEndpoint::Stdio {
        executable: std::env::current_exe()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        args: vec![],
        cwd: None,
        environment: Default::default(),
    };
    for authorization in [
        ConnectorAuthorization::Bearer,
        ConnectorAuthorization::OAuth {
            client_id: None,
            scopes: vec![],
        },
    ] {
        assert!(requirements::validate_access(&endpoint, &authorization).is_err());
    }
    requirements::validate_access(
        &endpoint,
        &ConnectorAuthorization::SecretFields {
            names: vec!["API_KEY".into()],
        },
    )
    .unwrap();
}

#[test]
fn obsolete_auth_only_service_and_endpoint_owning_instance_are_rejected() {
    let obsolete = serde_json::json!({"id":"service","name":"Service","pluginId":null,"authorization":{"type":"none"}});
    assert!(serde_json::from_value::<ConnectorDefinition>(obsolete).is_err());
    let obsolete = serde_json::json!({"id":"connection","connectorId":"service","name":"Work","revision":"old","accountId":null,"enabled":false,"tools":[],"endpoint":{"type":"streamable_http","url":"https://example.com/mcp","headers":{},"allowLocalNetwork":false}});
    assert!(serde_json::from_value::<McpConnection>(obsolete).is_err());
}
