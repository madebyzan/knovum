use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager, Runtime};

use super::{
    contracts::{MaintenanceReceipt, PendingOperation},
    file_publication,
};

const MAINTENANCE_DIR: &str = "maintenance";
const PENDING_FILE: &str = "pending-v1.json";
const RECEIPT_FILE: &str = "maintenance-receipt-v1.json";
const STATE_FILE_LIMIT: u64 = 64 * 1024;

pub struct AppPaths {
    pub config: PathBuf,
    pub data: PathBuf,
    pub maintenance: PathBuf,
}

impl AppPaths {
    pub fn resolve<R: Runtime>(app: &AppHandle<R>) -> Result<Self, String> {
        let config = app
            .path()
            .app_config_dir()
            .map_err(|error| error.to_string())?;
        let data = app
            .path()
            .app_data_dir()
            .map_err(|error| error.to_string())?;
        std::fs::create_dir_all(&config).map_err(|error| error.to_string())?;
        std::fs::create_dir_all(&data).map_err(|error| error.to_string())?;
        require_real_directory(&config, "configuration")?;
        require_real_directory(&data, "data")?;
        let maintenance = config.join(MAINTENANCE_DIR);
        std::fs::create_dir_all(&maintenance).map_err(|error| error.to_string())?;
        require_real_directory(&maintenance, "maintenance")?;
        Ok(Self {
            config,
            data,
            maintenance,
        })
    }

    pub fn pending(&self) -> PathBuf {
        self.maintenance.join(PENDING_FILE)
    }
    pub fn receipt(&self) -> PathBuf {
        self.maintenance.join(RECEIPT_FILE)
    }

    pub fn canonical_roots(&self) -> Result<(String, String), String> {
        let config = self
            .config
            .canonicalize()
            .map_err(|error| error.to_string())?;
        let data = self
            .data
            .canonicalize()
            .map_err(|error| error.to_string())?;
        Ok((path_string(config)?, path_string(data)?))
    }
}

fn path_string(path: PathBuf) -> Result<String, String> {
    path.into_os_string()
        .into_string()
        .map_err(|_| "Application data path is not valid UTF-8".into())
}

pub fn read_pending(paths: &AppPaths) -> Result<Option<PendingOperation>, String> {
    let path = paths.pending();
    if !entry_exists(&path)? {
        return Ok(None);
    }
    if entry_exists(&paths.receipt())? {
        read_receipt(paths)?.ok_or("Maintenance receipt disappeared while being inspected")?;
        std::fs::remove_file(path).map_err(|error| error.to_string())?;
        file_publication::sync_directory(&paths.maintenance)?;
        return Ok(None);
    }
    let bytes = read_bounded(&path, STATE_FILE_LIMIT)?;
    let pending = serde_json::from_slice(&bytes)
        .map_err(|error| format!("Pending maintenance request is invalid: {error}"))?;
    Ok(Some(pending))
}

pub fn clear_pending(paths: &AppPaths) -> Result<(), String> {
    let path = paths.pending();
    if entry_exists(&path)? {
        std::fs::remove_file(path).map_err(|error| error.to_string())?;
        file_publication::sync_directory(&paths.maintenance)?;
    }
    Ok(())
}

pub fn write_pending(paths: &AppPaths, pending: &PendingOperation) -> Result<(), String> {
    atomic_json(&paths.pending(), pending)
}

pub fn write_receipt(paths: &AppPaths, receipt: &MaintenanceReceipt) -> Result<(), String> {
    atomic_json(&paths.receipt(), receipt)
}

pub fn read_receipt(paths: &AppPaths) -> Result<Option<MaintenanceReceipt>, String> {
    let path = paths.receipt();
    if !entry_exists(&path)? {
        return Ok(None);
    }
    serde_json::from_slice(&read_bounded(&path, STATE_FILE_LIMIT)?)
        .map(Some)
        .map_err(|error| error.to_string())
}

pub fn acknowledge_receipt(paths: &AppPaths) -> Result<(), String> {
    let path = paths.receipt();
    if entry_exists(&path)? {
        std::fs::remove_file(path).map_err(|error| error.to_string())?;
        file_publication::sync_directory(&paths.maintenance)?;
    }
    Ok(())
}

fn atomic_json(path: &Path, value: &impl serde::Serialize) -> Result<(), String> {
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    file.write_all(&bytes).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    file_publication::replace_file(&temporary, path)
}

pub(super) fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Maintenance state must be a regular non-symbolic-link file".into());
    }
    let mut file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    if metadata.len() > limit {
        return Err(format!("Maintenance state exceeds the {limit}-byte limit"));
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > limit {
        return Err(format!("Maintenance state exceeds the {limit}-byte limit"));
    }
    Ok(bytes)
}

pub(super) fn entry_exists(path: &Path) -> Result<bool, String> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.to_string()),
    }
}

fn require_real_directory(path: &Path, name: &str) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(format!("Application {name} root must be a real directory"));
    }
    Ok(())
}
