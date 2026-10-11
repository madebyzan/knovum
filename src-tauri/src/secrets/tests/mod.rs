use super::validate_key;

#[test]
fn validates_product_secret_keys() {
    assert!(validate_key("ai.openai:team-token").is_ok());
    assert!(validate_key("").is_err());
    assert!(validate_key("contains space").is_err());
    assert!(validate_key("../escape").is_err());
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
