use std::{fs, path::PathBuf};

use sqlx::{migrate::Migrator, SqlitePool};
use tauri::{AppHandle, Manager};

static MIGRATOR: Migrator = sqlx::migrate!("src/secrets/migrations");

pub(super) async fn open(app: &AppHandle) -> Result<SqlitePool, String> {
    crate::backup::ready(app).await?;
    let path = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?
        .join("secrets.db");
    prepare_file(&path)?;
    let pool = crate::sqlite::connect_feature_database(app, "secrets.db").await?;
    MIGRATOR
        .run(&pool)
        .await
        .map_err(|error| format!("Failed to initialize secrets database: {error}"))?;
    Ok(pool)
}

fn prepare_file(path: &PathBuf) -> Result<(), String> {
    fs::create_dir_all(path.parent().ok_or("Secret database path has no parent")?)
        .map_err(|error| error.to_string())?;
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        options.mode(0o600);
        let file = options.open(path).map_err(|error| error.to_string())?;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|error| error.to_string())?;
    }
    #[cfg(not(unix))]
    options.open(path).map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) async fn get(pool: &SqlitePool, key: &str) -> Result<Option<String>, String> {
    sqlx::query_scalar("SELECT secret_value FROM secrets WHERE secret_key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|error| error.to_string())
}

pub(super) async fn is_configured(pool: &SqlitePool, key: &str) -> Result<bool, String> {
    sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM secrets WHERE secret_key = ? AND secret_value != '')",
    )
    .bind(key)
    .fetch_one(pool)
    .await
    .map_err(|error| error.to_string())
}

pub(super) async fn set(pool: &SqlitePool, key: &str, value: &str) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO secrets(secret_key, secret_value) VALUES (?, ?) \
         ON CONFLICT(secret_key) DO UPDATE SET secret_value = excluded.secret_value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) async fn delete(pool: &SqlitePool, key: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM secrets WHERE secret_key = ?")
        .bind(key)
        .execute(pool)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}
