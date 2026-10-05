use std::{
    collections::HashSet,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

use flate2::read::GzDecoder;
use tokio_util::sync::CancellationToken;

use crate::local_resources::preparation::RESOURCE_PREPARATION_CANCELLED;

pub(super) fn extract(
    source: &Path,
    destination: &Path,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let file = fs::File::open(source).map_err(|error| error.to_string())?;
    let mut archive = tar::Archive::new(GzDecoder::new(file));
    let mut entries = HashSet::new();
    let mut total = 0_u64;
    for entry in archive.entries().map_err(|error| error.to_string())? {
        require_active(cancellation)?;
        let mut entry = entry.map_err(|error| error.to_string())?;
        let path = entry
            .path()
            .map_err(|error| error.to_string())?
            .into_owned();
        if !safe_path(&path) || !entries.insert(path.clone()) || entries.len() > 100_000 {
            return Err("python-runtime-invalid: unsafe or duplicate archive path".into());
        }
        let size = entry.size();
        total = total
            .checked_add(size)
            .filter(|value| *value <= 16 * 1024 * 1024 * 1024)
            .ok_or("python-runtime-invalid: archive exceeds size limit")?;
        if size > 4 * 1024 * 1024 * 1024 {
            return Err("python-runtime-invalid: archive file exceeds size limit".into());
        }
        let output = destination.join(&path);
        match entry.header().entry_type() {
            tar::EntryType::Directory => {
                fs::create_dir_all(output).map_err(|error| error.to_string())?
            }
            tar::EntryType::Regular => {
                fs::create_dir_all(output.parent().ok_or("Archive file has no parent")?)
                    .map_err(|error| error.to_string())?;
                let mut file = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&output)
                    .map_err(|error| error.to_string())?;
                let mut buffer = [0_u8; 1024 * 1024];
                loop {
                    require_active(cancellation)?;
                    let count = entry.read(&mut buffer).map_err(|error| error.to_string())?;
                    if count == 0 {
                        break;
                    }
                    std::io::Write::write_all(&mut file, &buffer[..count])
                        .map_err(|error| error.to_string())?;
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = entry.header().mode().map_err(|error| error.to_string())? & 0o777;
                    fs::set_permissions(output, fs::Permissions::from_mode(mode))
                        .map_err(|error| error.to_string())?;
                }
            }
            _ => {
                return Err(
                    "python-runtime-invalid: archive links and special files are forbidden".into(),
                )
            }
        }
    }
    Ok(())
}

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

pub(super) fn require_active(cancellation: &CancellationToken) -> Result<(), String> {
    if cancellation.is_cancelled() {
        Err(RESOURCE_PREPARATION_CANCELLED.into())
    } else {
        Ok(())
    }
}

pub(super) async fn unpack(
    source: PathBuf,
    destination: PathBuf,
    cancellation: CancellationToken,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || extract(&source, &destination, &cancellation))
        .await
        .map_err(|error| error.to_string())?
}

#[cfg(test)]
mod tests;
