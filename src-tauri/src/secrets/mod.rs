mod vault;

use std::{path::PathBuf, sync::OnceLock};

use tauri::{plugin::TauriPlugin, AppHandle, Manager, Wry};

use self::vault::SecretVault;

const MAX_SECRET_KEY_LENGTH: usize = 128;
static VAULT_PATH: OnceLock<PathBuf> = OnceLock::new();
static VAULT: OnceLock<std::sync::Mutex<VaultSession>> = OnceLock::new();

enum VaultSession {
    Locked,
    Ready(SecretVault),
    Failed(String),
}

pub(crate) fn install(app: &AppHandle) -> Result<(), String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("secrets.vault");
    VAULT_PATH
        .set(path)
        .map_err(|_| "secret vault path was initialized twice".to_owned())?;
    VAULT
        .set(std::sync::Mutex::new(VaultSession::Locked))
        .map_err(|_| "secret vault state was initialized twice".to_owned())
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

pub(crate) fn read_secret(secret_key: &str) -> Result<Option<String>, String> {
    validate_key(secret_key)?;
    with_vault(|vault| Ok(vault.get(secret_key)))
}

#[tauri::command]
pub fn get(secret_key: String) -> Result<Option<String>, String> {
    read_secret(&secret_key)
}

#[tauri::command]
pub fn set(secret_key: String, secret: String) -> Result<(), String> {
    validate_key(&secret_key)?;
    with_vault(|vault| vault.set(secret_key, secret))
}

#[tauri::command]
pub fn delete(secret_key: String) -> Result<(), String> {
    validate_key(&secret_key)?;
    with_vault(|vault| vault.delete(&secret_key))
}

pub fn plugin() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("secrets")
        .invoke_handler(tauri::generate_handler![get, set, delete])
        .build()
}

fn with_vault<T>(action: impl FnOnce(&mut SecretVault) -> Result<T, String>) -> Result<T, String> {
    let path = VAULT_PATH
        .get()
        .ok_or_else(|| "secret vault is not initialized".to_owned())?;
    let state = VAULT
        .get()
        .ok_or_else(|| "secret vault state is not initialized".to_owned())?;
    let mut guard = state
        .lock()
        .map_err(|_| "secret vault lock is poisoned".to_owned())?;
    if matches!(*guard, VaultSession::Locked) {
        *guard = match SecretVault::open(path.clone()) {
            Ok(vault) => VaultSession::Ready(vault),
            Err(error) => VaultSession::Failed(error),
        };
    }
    match &mut *guard {
        VaultSession::Ready(vault) => action(vault),
        VaultSession::Failed(error) => Err(error.clone()),
        VaultSession::Locked => unreachable!("secret vault session was resolved"),
    }
}

#[cfg(test)]
mod tests;
