use sqlx::{Sqlite, SqlitePool, Transaction};
use tauri::{AppHandle, Runtime};
use tokio::sync::OnceCell;

static SPACE_DATABASE: OnceCell<SqlitePool> = OnceCell::const_new();
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/spaces");
mod document_reset;

pub async fn pool<R: Runtime>(app: &AppHandle<R>) -> Result<&'static SqlitePool, String> {
    SPACE_DATABASE
        .get_or_try_init(|| async {
            let pool = crate::sqlite::connect_feature_database(app, "spaces.db").await?;
            migrate(&pool)
                .await
                .map_err(|error| format!("Failed to migrate Spaces database: {error}"))?;
            document_reset::complete(&pool).await?;
            crate::spaces::assets::reclaim_orphan_managed_assets(app, &pool).await?;
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
