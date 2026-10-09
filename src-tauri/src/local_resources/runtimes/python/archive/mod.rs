#[cfg(any(not(debug_assertions), test))]
use std::{
    collections::HashSet,
    fs,
    io::Read,
    path::{Component, Path},
};

#[cfg(any(not(debug_assertions), test))]
use flate2::read::GzDecoder;
use tokio_util::sync::CancellationToken;

use super::RuntimeError;

#[cfg(any(not(debug_assertions), test))]
pub(super) fn extract(
    source: &Path,
    destination: &Path,
    cancellation: &CancellationToken,
) -> Result<(), RuntimeError> {
    let file = fs::File::open(source)?;
    let mut archive = tar::Archive::new(GzDecoder::new(file));
    let mut entries = HashSet::new();
    let mut total = 0_u64;
    for entry in archive.entries()? {
        require_active(cancellation)?;
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        if !safe_path(&path) || !entries.insert(path.clone()) || entries.len() > 100_000 {
            return Err(RuntimeError::InvalidPublication(
                "unsafe or duplicate archive path".into(),
            ));
        }
        let size = entry.size();
        total = total
            .checked_add(size)
            .filter(|value| *value <= 16 * 1024 * 1024 * 1024)
            .ok_or_else(|| RuntimeError::InvalidPublication("archive exceeds size limit".into()))?;
        if size > 4 * 1024 * 1024 * 1024 {
            return Err(RuntimeError::InvalidPublication(
                "archive file exceeds size limit".into(),
            ));
        }
        let output = destination.join(&path);
        match entry.header().entry_type() {
            tar::EntryType::Directory => fs::create_dir_all(output)?,
            tar::EntryType::Regular => {
                fs::create_dir_all(output.parent().ok_or_else(|| {
                    RuntimeError::InvalidPublication("Archive file has no parent".into())
                })?)?;
                let mut file = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&output)?;
                let mut buffer = [0_u8; 1024 * 1024];
                loop {
                    require_active(cancellation)?;
                    let count = entry.read(&mut buffer)?;
                    if count == 0 {
                        break;
                    }
                    std::io::Write::write_all(&mut file, &buffer[..count])?;
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = entry.header().mode()? & 0o777;
                    fs::set_permissions(output, fs::Permissions::from_mode(mode))?;
                }
            }
            _ => {
                return Err(RuntimeError::InvalidPublication(
                    "archive links and special files are forbidden".into(),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(any(not(debug_assertions), test))]
fn safe_path(path: &Path) -> bool {
    let value = path.to_string_lossy();
    let value = value.strip_suffix('/').unwrap_or(&value);
    !value.is_empty()
        && !value.contains(['\\', ':'])
        && value
            .split('/')
            .all(|part| !matches!(part, "." | ".." | ""))
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

pub(super) fn require_active(cancellation: &CancellationToken) -> Result<(), RuntimeError> {
    if cancellation.is_cancelled() {
        Err(RuntimeError::Cancelled)
    } else {
        Ok(())
    }
}

#[cfg(not(debug_assertions))]
pub(super) async fn unpack(
    source: std::path::PathBuf,
    destination: std::path::PathBuf,
    cancellation: CancellationToken,
) -> Result<(), RuntimeError> {
    tokio::task::spawn_blocking(move || extract(&source, &destination, &cancellation))
        .await
        .map_err(|error| RuntimeError::Infrastructure(error.to_string()))?
}

#[cfg(test)]
mod tests;
