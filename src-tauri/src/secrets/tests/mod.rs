use super::{is_webview_readable_key, validate_key};

#[test]
fn validates_product_secret_keys() {
    assert!(validate_key("ai.openai:team-token").is_ok());
    assert!(validate_key("").is_err());
    assert!(validate_key("contains space").is_err());
    assert!(validate_key("../escape").is_err());
}

#[test]
fn restricts_webview_secret_reads_to_space_credential_nodes() {
    assert!(is_webview_readable_key("spaces.credential:node-123"));
    assert!(!is_webview_readable_key("spaces.credential:"));
    assert!(!is_webview_readable_key(
        "integrations.ai.connections.primary.api-key"
    ));
    assert!(!is_webview_readable_key(
        "integrations.storage.object-storage.secret-access-key"
    ));
    assert!(!is_webview_readable_key("agent-extension:github"));
}

#[tokio::test]
async fn secret_database_uses_delete_journal_without_wal_sidecars() {
    let dir = std::env::temp_dir().join(format!("knovum-secrets-test-{}", uuid::Uuid::new_v4()));
    let path = dir.join("secrets.db");
    let pool = super::store::open_at(&path).await.unwrap();
    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(journal_mode.to_ascii_lowercase(), "delete");
    super::store::set(&pool, "integrations.ai.key", "secret-token")
        .await
        .unwrap();
    assert!(!dir.join("secrets.db-wal").exists());
    assert!(!dir.join("secrets.db-shm").exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
    pool.close().await;
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn get_command_rejects_non_credential_node_namespaces() {
    let error = super::get("integrations.ai.connections.primary.api-key".into())
        .await
        .unwrap_err();
    assert_eq!(error.code, "forbidden");
}

#[tokio::test]
async fn configuration_state_tracks_nonempty_credentials_without_returning_them() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("src/secrets/migrations")
        .run(&pool)
        .await
        .unwrap();
    assert!(!super::store::is_configured(&pool, "ai.key").await.unwrap());
    super::store::set(&pool, "ai.key", "").await.unwrap();
    assert!(!super::store::is_configured(&pool, "ai.key").await.unwrap());
    super::store::set(&pool, "ai.key", "private-value")
        .await
        .unwrap();
    assert!(super::store::is_configured(&pool, "ai.key").await.unwrap());
    super::store::delete(&pool, "ai.key").await.unwrap();
    assert!(!super::store::is_configured(&pool, "ai.key").await.unwrap());
    pool.close().await;
    assert!(super::store::is_configured(&pool, "ai.key").await.is_err());
}
