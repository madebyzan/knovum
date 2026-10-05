use super::update_state;
use sqlx::SqlitePool;

async fn pool() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("CREATE TABLE spaces_voice_profiles(id TEXT PRIMARY KEY, archived_at INTEGER, updated_at INTEGER NOT NULL)")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_voice_profiles VALUES('voice',NULL,1)")
        .execute(&pool)
        .await
        .unwrap();
    pool
}

#[tokio::test]
async fn archive_and_unarchive_preserve_profile_identity() {
    let pool = pool().await;
    update_state(&pool, "voice", 1, true).await.unwrap();
    let (archived_at, updated_at): (Option<i64>, i64) =
        sqlx::query_as("SELECT archived_at,updated_at FROM spaces_voice_profiles WHERE id='voice'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(archived_at, Some(updated_at));
    update_state(&pool, "voice", updated_at, false)
        .await
        .unwrap();
    let archived_at: Option<i64> =
        sqlx::query_scalar("SELECT archived_at FROM spaces_voice_profiles WHERE id='voice'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(archived_at, None);
}

#[tokio::test]
async fn stale_archive_request_cannot_change_profile_state() {
    let pool = pool().await;
    assert!(update_state(&pool, "voice", 2, true).await.is_err());
    let state: (Option<i64>, i64) =
        sqlx::query_as("SELECT archived_at,updated_at FROM spaces_voice_profiles WHERE id='voice'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(state, (None, 1));
}
