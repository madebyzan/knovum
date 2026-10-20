use crate::foundation::app_error::AppError;
mod store;

use std::sync::OnceLock;

use sqlx::SqlitePool;
use tauri::{plugin::TauriPlugin, AppHandle, Wry};
use tokio::sync::OnceCell;

const MAX_SECRET_KEY_LENGTH: usize = 128;
static APP: OnceLock<AppHandle> = OnceLock::new();
static DATABASE: OnceCell<SqlitePool> = OnceCell::const_new();

pub(crate) fn install(app: &AppHandle) -> Result<(), String> {
    APP.set(app.clone())
        .map_err(|_| "secret database was initialized twice".to_owned())
}

fn validate_key(secret_key: &str) -> Result<(), String> {
    let valid = !secret_key.is_empty()
        && secret_key.len() <= MAX_SECRET_KEY_LENGTH
        && secret_key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte));

    if valid {
        Ok(())
    } else {
        Err("secret key must contain only letters, numbers, '.', '_', ':', or '-'".into())
    }
}

async fn database() -> Result<&'static SqlitePool, String> {
    let app = APP
        .get()
        .ok_or_else(|| "secret database is not initialized".to_owned())?;
    DATABASE.get_or_try_init(|| store::open(app)).await
}

pub(crate) async fn read_secret(secret_key: &str) -> Result<Option<String>, String> {
    validate_key(secret_key)?;
    store::get(database().await?, secret_key).await
}

pub(crate) async fn write_secret(secret_key: &str, secret: &str) -> Result<(), String> {
    validate_key(secret_key)?;
    store::set(database().await?, secret_key, secret).await
}

pub(crate) async fn remove_secret(secret_key: &str) -> Result<(), String> {
    validate_key(secret_key)?;
    store::delete(database().await?, secret_key).await
}

#[tauri::command]
pub async fn get(secret_key: String) -> Result<Option<String>, AppError> {
    Ok((read_secret(&secret_key).await)?)
}

#[tauri::command]
pub async fn inspect(secret_key: String) -> Result<bool, AppError> {
    validate_key(&secret_key)?;
    Ok(store::is_configured(database().await?, &secret_key).await?)
}

#[tauri::command]
pub async fn set(secret_key: String, secret: String) -> Result<(), AppError> {
    Ok(write_secret(&secret_key, &secret).await?)
}

#[tauri::command]
pub async fn delete(secret_key: String) -> Result<(), AppError> {
    Ok(remove_secret(&secret_key).await?)
}

pub fn plugin() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("secrets")
        .invoke_handler(tauri::generate_handler![get, inspect, set, delete])
        .build()
}

#[cfg(test)]
mod tests;
