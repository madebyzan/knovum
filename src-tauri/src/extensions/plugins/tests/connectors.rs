use crate::extensions::{connectors::definitions, contracts::*, database, mcp::connections};
use serde_json::json;
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
fn component(name: &str) -> (PluginConnectorDeclaration, McpEndpoint) {
    (
        PluginConnectorDeclaration {
            component_name: name.into(),
            name: "GitHub".into(),
            description: "Read issues".into(),
            configuration: json!({}),
            authorization: ConnectorAuthorization::None,
        },
        McpEndpoint::StreamableHttp {
            url: "https://example.com/mcp".into(),
            headers: Default::default(),
            allow_local_network: false,
        },
    )
}
async fn publish(
    pool: &SqlitePool,
    revision: &str,
    components: &[(PluginConnectorDeclaration, McpEndpoint)],
) {
    let mut tx = pool.begin().await.unwrap();
    super::publish_on(&mut tx, "plugin", revision, components)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

#[tokio::test]
async fn plugin_upgrade_preserves_definition_identity_all_instances_and_independent_preferences() {
    let pool = pool().await;
    publish(&pool, "one", &[component("github")]).await;
    let old = definitions::list(&pool).await.unwrap().remove(0);
    let mut first = connections::list(&pool).await.unwrap().remove(0);
    assert!(!first.enabled);
    first.name = "Work".into();
    first.enabled = true;
    first.tools = vec![McpToolPermission {
        name: "list_issues".into(),
        schema_digest: "reviewed".into(),
        read_only: true,
        policy: McpToolPolicy::Allow,
    }];
    sqlx::query("UPDATE connections SET document=$1 WHERE id=$2")
        .bind(database::encode(&first).unwrap())
        .bind(&first.id)
        .execute(&pool)
        .await
        .unwrap();
    let mut second = first.clone();
    second.id = "second".into();
    second.name = "Personal".into();
    second.enabled = false;
    second.tools.clear();
    sqlx::query("INSERT INTO connections(id,document) VALUES($1,$2)")
        .bind(&second.id)
        .bind(database::encode(&second).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    publish(&pool, "two", &[component("github")]).await;
    let current = definitions::list(&pool).await.unwrap();
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].id, old.id);
    assert_eq!(current[0].revision, "two");
    assert!(definitions::require_current(&pool, &old).await.is_err());
    let current_first = connections::load(&pool, &first.id).await.unwrap();
    let current_second = connections::load(&pool, &second.id).await.unwrap();
    assert_eq!(connections::list(&pool).await.unwrap().len(), 2);
    assert_eq!(current_first.name, "Work");
    assert!(current_first.enabled);
    assert_eq!(current_first.tools.len(), 1);
    assert_ne!(current_first.revision, first.revision);
    assert_eq!(current_second.name, "Personal");
    assert!(!current_second.enabled);
    assert!(current_second.tools.is_empty());
}

#[tokio::test]
async fn removed_plugin_component_retires_its_instances_without_recreating_user_deleted_instances()
{
    let pool = pool().await;
    publish(&pool, "one", &[component("github"), component("retired")]).await;
    let old = definitions::list(&pool).await.unwrap();
    let retained = old
        .iter()
        .find(|definition| definition.component_name.as_deref() == Some("github"))
        .unwrap();
    sqlx::query("DELETE FROM connections WHERE json_extract(document,'$.connectorId')=$1")
        .bind(&retained.id)
        .execute(&pool)
        .await
        .unwrap();
    publish(&pool, "two", &[component("github")]).await;
    let current = definitions::list(&pool).await.unwrap();
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].id, retained.id);
    assert!(connections::list(&pool).await.unwrap().is_empty());
}
