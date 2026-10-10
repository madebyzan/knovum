use super::create;
use crate::extensions::{
    connectors::definitions,
    contracts::{ConnectorAuthorization, ConnectorDefinition, McpEndpoint},
    mcp::connections,
};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

async fn pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("migrations/extensions")
        .run(&pool)
        .await
        .unwrap();
    pool
}
fn definition() -> ConnectorDefinition {
    ConnectorDefinition {
        id: "renderer-supplied".into(),
        revision: "renderer-supplied".into(),
        name: "Research".into(),
        description: "Search authorized sources".into(),
        plugin_id: Some("renderer-supplied".into()),
        component_name: Some("renderer-supplied".into()),
        endpoint: Some(McpEndpoint::StreamableHttp {
            url: "https://example.com/mcp".into(),
            headers: Default::default(),
            allow_local_network: false,
        }),
        authorization: ConnectorAuthorization::Bearer,
    }
}
#[tokio::test]
async fn creates_a_custom_service_and_disabled_first_connection_together() {
    let pool = pool().await;
    let connection = create(&pool, definition()).await.unwrap();
    let connector = definitions::load(&pool, &connection.connector_id)
        .await
        .unwrap();
    assert_ne!(connector.id, "renderer-supplied");
    assert!(connector.plugin_id.is_none());
    assert!(connector.component_name.is_none());
    assert_eq!(connection.name, connector.name);
    assert!(!connection.enabled);
    assert!(connection.account_id.is_none());
    assert!(connection.tools.is_empty());
    assert_eq!(connections::list(&pool).await.unwrap().len(), 1);
}
#[tokio::test]
async fn a_failed_connection_insert_rolls_back_the_service_definition() {
    let pool = pool().await;
    sqlx::query("CREATE TRIGGER reject_connection BEFORE INSERT ON connections BEGIN SELECT RAISE(FAIL,'test rejection'); END;").execute(&pool).await.unwrap();
    assert!(create(&pool, definition()).await.is_err());
    assert!(definitions::list(&pool).await.unwrap().is_empty());
    assert!(connections::list(&pool).await.unwrap().is_empty());
}
#[tokio::test]
async fn malformed_service_does_not_create_either_entity() {
    let pool = pool().await;
    let mut malformed = definition();
    malformed.endpoint = None;
    assert!(create(&pool, malformed).await.is_err());
    assert!(definitions::list(&pool).await.unwrap().is_empty());
    assert!(connections::list(&pool).await.unwrap().is_empty());
}

#[tokio::test]
async fn service_description_is_optional_but_name_is_required() {
    let pool = pool().await;
    let mut custom = definition();
    custom.description.clear();
    assert!(create(&pool, custom.clone()).await.is_ok());
    custom.name = " ".into();
    assert_eq!(
        create(&pool, custom).await.unwrap_err(),
        "connector-name-required"
    );
    assert_eq!(definitions::list(&pool).await.unwrap().len(), 1);
}
