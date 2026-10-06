use super::ExtensionState;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};

pub(super) async fn pool(app: &AppHandle) -> Result<SqlitePool, String> {
    let state = app.state::<ExtensionState>();
    state.database.get_or_try_init(|| async {
        let pool = crate::sqlite::connect_feature_database(app, "extensions.db").await?;
        sqlx::migrate!("migrations/extensions").run(&pool).await
            .map_err(|error| format!("extensions-database-invalid: {error}; reset obsolete pre-release extension data explicitly"))?;
        Ok::<_,String>(pool)
    }).await?;
    Ok(state
        .database
        .get()
        .expect("initialized extension database")
        .clone())
}

pub(super) fn encode(value: &impl serde::Serialize) -> Result<String, String> {
    serde_json::to_string(value).map_err(|error| error.to_string())
}

pub(super) fn decode<T: serde::de::DeserializeOwned>(document: &str) -> Result<T, String> {
    serde_json::from_str(document).map_err(|error| format!("extensions-data-invalid: {error}"))
}
