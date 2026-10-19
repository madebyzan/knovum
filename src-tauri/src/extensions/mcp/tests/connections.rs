use crate::extensions::contracts::{ConnectorAuthorization, ConnectorDefinition};

#[tokio::test]
async fn packaged_connection_enablement_follows_the_package_and_missing_owners_fail_visibly() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("src/extensions/migrations")
        .run(&pool)
        .await
        .unwrap();
    let plugin = serde_json::json!({
        "id":"package","enabled":true,"builtin":false,"publication":null,
        "source":{"type":"directory","path":"/package"},
        "revision":{"digest":"one","sourceRevision":"one","components":[],"connectors":[],"fileCount":1,"byteLength":1,
            "manifest":{"$schema":"https://test/plugin.schema.json","name":"Knowledge"}}
    });
    let connector = ConnectorDefinition {
        id: "service".into(),
        revision: "one".into(),
        name: "Sources".into(),
        description: "Evidence".into(),
        plugin_id: Some("package".into()),
        component_name: Some("sources".into()),
        endpoint: None,
        authorization: ConnectorAuthorization::None,
    };
    sqlx::query("INSERT INTO plugins(id,document) VALUES('package',$1)")
        .bind(sqlx::types::Json(plugin))
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO connectors(id,document) VALUES('service',$1)")
        .bind(sqlx::types::Json(connector))
        .execute(&pool)
        .await
        .unwrap();
    let mut tx = pool.begin().await.unwrap();
    let connection = super::create_on(&mut tx, "service".into(), "Sources".into())
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert!(!connection.enabled);
    assert!(super::load(&pool, &connection.id).await.unwrap().enabled);
    assert!(super::list(&pool).await.unwrap()[0].enabled);
    sqlx::query("UPDATE plugins SET document=json_set(document,'$.enabled',json('false'))")
        .execute(&pool)
        .await
        .unwrap();
    assert!(!super::load(&pool, &connection.id).await.unwrap().enabled);
    sqlx::query("DELETE FROM plugins")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        super::load(&pool, &connection.id).await.unwrap_err(),
        "mcp-plugin-missing"
    );
    assert_eq!(super::list(&pool).await.unwrap_err(), "mcp-plugin-missing");
}
