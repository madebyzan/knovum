use sqlx::SqlitePool;
use tauri::{AppHandle, Runtime};
use tokio::sync::OnceCell;

static DATABASE: OnceCell<SqlitePool> = OnceCell::const_new();
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("src/ai/usage/migrations");

pub async fn initialize<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let _ = pool(app).await?;
    Ok(())
}

pub async fn pool<R: Runtime>(app: &AppHandle<R>) -> Result<&'static SqlitePool, String> {
    DATABASE
        .get_or_try_init(|| async {
            let pool = crate::sqlite::connect_feature_database(app, "ai-usage.db").await?;
            MIGRATOR
                .run(&pool)
                .await
                .map_err(|error| format!("Failed to migrate AI usage database: {error}"))?;
            Ok(pool)
        })
        .await
}

pub fn initialized_pool() -> Result<&'static SqlitePool, String> {
    DATABASE
        .get()
        .ok_or_else(|| "AI usage database is not initialized".to_owned())
}
