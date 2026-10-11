use crate::extensions::{
    connectors::definitions,
    contracts::{
        ConnectorAuthorization, ConnectorDefinition, McpEndpoint, McpToolPermission, McpToolPolicy,
        PluginConnectorConfiguration, PluginConnectorDeclaration,
    },
    database,
    mcp::connections,
};
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
    let plugin = json!({
        "id":"plugin","enabled":true,"builtin":false,"publication":null,
        "source":{"type":"directory","path":"/plugin"},
        "revision":{"digest":"one","sourceRevision":"one","components":[],"connectors":[],"fileCount":1,"byteLength":1,
            "manifest":{"$schema":"https://test/plugin.schema.json","name":"Sources"}}
    });
    sqlx::query("INSERT INTO plugins(id,document) VALUES('plugin',$1)")
        .bind(sqlx::types::Json(plugin))
        .execute(&pool)
        .await
        .unwrap();
    pool
}
fn component(name: &str) -> (PluginConnectorDeclaration, Option<McpEndpoint>) {
    (
        PluginConnectorDeclaration {
            component_name: name.into(),
            name: "GitHub".into(),
            description: "Read issues".into(),
            configuration: json!({}),
            authorization: ConnectorAuthorization::None,
        },
        Some(McpEndpoint::StreamableHttp {
            url: "https://example.com/mcp".into(),
            headers: Default::default(),
            allow_local_network: false,
        }),
    )
}
async fn publish(
    pool: &SqlitePool,
    revision: &str,
    components: &[(PluginConnectorDeclaration, Option<McpEndpoint>)],
) {
    let mut tx = pool.begin().await.unwrap();
    super::publish_on(&mut tx, "plugin", revision, components)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

#[tokio::test]
async fn plugin_upgrade_preserves_bindings_and_grants_with_package_activation() {
    let pool = pool().await;
    publish(&pool, "one", &[component("github")]).await;
    let old = definitions::list(&pool).await.unwrap().remove(0);
    let mut first = connections::list(&pool).await.unwrap().remove(0);
    assert!(first.enabled);
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
    assert!(current_second.enabled);
    assert!(current_second.tools.is_empty());
}

#[tokio::test]
async fn removed_plugin_component_retires_connections_without_recreating_deleted_connections() {
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

#[tokio::test]
async fn unconfigured_stdio_component_preserves_package_activation_without_an_endpoint() {
    let pool = pool().await;
    let (mut declaration, _) = component("local-server");
    declaration.configuration = json!({"type":"stdio","command":"node"});
    publish(&pool, "one", &[(declaration, None)]).await;
    let definition = definitions::list(&pool).await.unwrap().remove(0);
    assert!(definition.configured_endpoint().is_err());
    let connection = connections::list(&pool).await.unwrap().remove(0);
    assert!(connection.enabled);
    assert_eq!(connection.connector_id, definition.id);
}

#[test]
fn installed_program_binding_survives_same_command_and_expires_when_command_changes() {
    let (mut declaration, _) = component("local-server");
    declaration.configuration = json!({"type":"stdio","command":"node","args":["one"]});
    let endpoint = Some(McpEndpoint::Stdio {
        executable: "/installed/node".into(),
        args: vec!["one".into()],
        cwd: None,
        environment: Default::default(),
    });
    let definition = ConnectorDefinition {
        id: "service".into(),
        revision: "one".into(),
        name: declaration.name.clone(),
        description: declaration.description.clone(),
        plugin_id: Some("plugin".into()),
        component_name: Some(declaration.component_name.clone()),
        endpoint,
        authorization: ConnectorAuthorization::None,
    };
    let mut changed = declaration.clone();
    changed.configuration["args"] = json!(["two"]);
    assert!(
        matches!(super::retained_configuration(&changed, Some(&declaration), Some(&definition)), Some(PluginConnectorConfiguration::Stdio { executable }) if executable == "/installed/node")
    );
    changed.configuration["command"] = json!("python");
    assert!(
        super::retained_configuration(&changed, Some(&declaration), Some(&definition)).is_none()
    );
}

#[test]
fn network_permission_is_not_transferred_to_a_different_service_url() {
    let (mut declaration, _) = component("remote");
    declaration.configuration = json!({"type":"streamable-http","url":"https://example.com/mcp"});
    let definition = ConnectorDefinition {
        id: "service".into(),
        revision: "one".into(),
        name: declaration.name.clone(),
        description: declaration.description.clone(),
        plugin_id: Some("plugin".into()),
        component_name: Some(declaration.component_name.clone()),
        endpoint: Some(McpEndpoint::StreamableHttp {
            url: "https://example.com/mcp".into(),
            headers: Default::default(),
            allow_local_network: true,
        }),
        authorization: ConnectorAuthorization::None,
    };
    assert!(matches!(
        super::retained_configuration(&declaration, Some(&declaration), Some(&definition)),
        Some(PluginConnectorConfiguration::StreamableHttp {
            allow_local_network: true
        })
    ));
    let mut changed = declaration.clone();
    changed.configuration["url"] = json!("https://other.example.com/mcp");
    assert!(
        super::retained_configuration(&changed, Some(&declaration), Some(&definition)).is_none()
    );
}
