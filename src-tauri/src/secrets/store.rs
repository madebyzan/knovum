use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use tauri::{AppHandle, Manager};

static MIGRATOR: Migrator = sqlx::migrate!("src/secrets/migrations");

pub(super) async fn open(app: &AppHandle) -> Result<SqlitePool, String> {
    crate::backup::ready(app).await?;
    let path = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?
        .join("secrets.db");
    open_at(&path).await
}

pub(super) async fn open_at(path: &Path) -> Result<SqlitePool, String> {
    prepare_file(path)?;
    // Use DELETE journal mode rather than WAL so plaintext credentials never linger in
    // default-umask `-wal` sidecar files on Unix.
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Delete)
        .synchronous(SqliteSynchronous::Full)
        .busy_timeout(Duration::from_secs(5))
        .pragma("temp_store", "MEMORY");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|error| error.to_string())?;
    MIGRATOR
        .run(&pool)
        .await
        .map_err(|error| format!("Failed to initialize secrets database: {error}"))?;
    remove_stale_wal_sidecars(path);
    Ok(pool)
}

fn prepare_file(path: &Path) -> Result<(), String> {
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

fn remove_stale_wal_sidecars(path: &Path) {
    for suffix in ["-wal", "-shm"] {
        let mut sidecar: OsString = path.as_os_str().into();
        sidecar.push(suffix);
        let sidecar = PathBuf::from(sidecar);
        if let Err(error) = fs::remove_file(&sidecar) {
            if error.kind() != std::io::ErrorKind::NotFound {
                log::warn!("failed to remove stale secrets sidecar: {error}");
            }
        }
    }
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
