use super::accounts;
use crate::extensions::{
    contracts::{ConnectorAccount, ConnectorAuthorization},
    database,
};
use serde_json::json;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

async fn fixture() -> (SqlitePool, ConnectorAccount) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("src/extensions/migrations")
        .run(&pool)
        .await
        .unwrap();
    let account = ConnectorAccount {
        id: "account".into(),
        revision: "first-authorization".into(),
        connector_id: "connector".into(),
        display_name: "Account".into(),
        resource: "https://example.com/mcp".into(),
        issuer: None,
        scopes: vec![],
        authorization: ConnectorAuthorization::Bearer,
        enabled: true,
        cleanup_pending: false,
    };
    accounts::save(&pool, &account).await.unwrap();
    for (id, account_id) in [("bound", "account"), ("unrelated", "other-account")] {
        let document = json!({"id":id,"accountId":account_id,"revision":"reviewed","tools":[{"name":"publish"}]});
        sqlx::query("INSERT INTO connections(id,document) VALUES($1,$2)")
            .bind(id)
            .bind(document.to_string())
            .execute(&pool)
            .await
            .unwrap();
    }
    (pool, account)
}

async fn connection(pool: &SqlitePool, id: &str) -> serde_json::Value {
    let document: String = sqlx::query_scalar("SELECT document FROM connections WHERE id=$1")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap();
    database::decode(&document).unwrap()
}

#[tokio::test]
async fn reauthorization_invalidates_frozen_identity_and_bound_tool_review_atomically() {
    let (pool, frozen) = fixture().await;
    accounts::set_status(&pool, &frozen.id, false, false)
        .await
        .unwrap();
    let previous = accounts::load(&pool, &frozen.id).await.unwrap();
    let mut renewed = frozen.clone();
    renewed.revision = "second-authorization".into();
    accounts::commit_authorization(&pool, &renewed, &previous)
        .await
        .unwrap();
    assert!(accounts::require_current(&pool, &frozen).await.is_err());
    accounts::require_current(&pool, &renewed).await.unwrap();
    let bound = connection(&pool, "bound").await;
    assert_eq!(bound["tools"], json!([]));
    assert_ne!(bound["revision"], "reviewed");
    let mut tx = pool.begin().await.unwrap();
    assert!(
        crate::extensions::mcp::connections::require_revision_on(&mut tx, "bound", "reviewed")
            .await
            .is_err()
    );
    tx.rollback().await.unwrap();
    let unrelated = connection(&pool, "unrelated").await;
    assert_eq!(unrelated["revision"], "reviewed");
    assert_eq!(unrelated["tools"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn concurrent_disconnect_prevents_commit_and_preserves_tool_review() {
    let (pool, mut renewed) = fixture().await;
    let previous = accounts::load(&pool, &renewed.id).await.unwrap();
    renewed.revision = "second-authorization".into();
    accounts::set_status(&pool, &renewed.id, false, true)
        .await
        .unwrap();
    assert_eq!(
        accounts::commit_authorization(&pool, &renewed, &previous)
            .await
            .unwrap_err(),
        "connector-account-revoked-during-authorization"
    );
    let current = accounts::load(&pool, &renewed.id).await.unwrap();
    assert!(!current.enabled);
    assert!(current.cleanup_pending);
    assert_eq!(current.revision, previous.revision);
    assert!(accounts::require_current(&pool, &previous).await.is_err());
    assert_eq!(
        connection(&pool, "bound").await["tools"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn disabled_and_cleanup_pending_accounts_cannot_dispatch() {
    let (pool, frozen) = fixture().await;
    for (enabled, cleanup_pending) in [(false, false), (true, true)] {
        accounts::set_status(&pool, &frozen.id, enabled, cleanup_pending)
            .await
            .unwrap();
        assert!(accounts::require_current(&pool, &frozen).await.is_err());
    }
}
