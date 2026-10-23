use sqlx::{Sqlite, SqlitePool, Transaction};
use tauri::{AppHandle, Runtime};
use tokio::sync::OnceCell;

static SPACE_DATABASE: OnceCell<SqlitePool> = OnceCell::const_new();
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("src/spaces/db/migrations");
pub async fn pool<R: Runtime>(app: &AppHandle<R>) -> Result<&'static SqlitePool, String> {
    SPACE_DATABASE
        .get_or_try_init(|| async {
            let pool = crate::sqlite::connect_feature_database(app, "spaces.db").await?;
            migrate(&pool)
                .await
                .map_err(|error| match error {
                    sqlx::migrate::MigrateError::VersionMissing(version) => format!(
                        "spaces-database-newer: spaces.db records migration {version}, which this build does not carry. The library was written by a newer release of Knovum; install that release again. Nothing was changed."
                    ),
                    sqlx::migrate::MigrateError::VersionMismatch(version) => format!(
                        "spaces-database-migration-edited: migration {version} no longer matches the one applied to spaces.db. A shipped migration must never be edited. Nothing was changed."
                    ),
                    _ => format!("Failed to initialize Spaces database: {error}"),
                })?;
            crate::sqlite::optimize(&pool).await;
            Ok(pool)
        })
        .await
}

pub(crate) async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await
}

pub fn sql_error(error: sqlx::Error) -> String {
    error.to_string()
}

pub async fn begin_write(pool: &SqlitePool) -> Result<Transaction<'_, Sqlite>, String> {
    pool.begin_with("BEGIN IMMEDIATE").await.map_err(sql_error)
}

pub fn json_error(error: serde_json::Error) -> String {
    error.to_string()
}

pub fn require_one(executed: sqlx::sqlite::SqliteQueryResult, message: &str) -> Result<(), String> {
    if executed.rows_affected() == 1 {
        Ok(())
    } else {
        Err(message.to_owned())
    }
}

#[cfg(test)]
mod tests;
