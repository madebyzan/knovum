use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use tokio_util::sync::CancellationToken;

const MAX_DENO_BINARY_BYTES: u64 = 192 * 1024 * 1024;

pub(super) async fn extract(
    archive_path: PathBuf,
    binary: PathBuf,
    binary_name: String,
    cancellation: CancellationToken,
) -> Result<(), String> {
    crate::blocking::run(move || {
        extract_blocking(&archive_path, &binary, &binary_name, &cancellation)
    })
    .await
}

fn extract_blocking(
    archive_path: &Path,
    binary: &Path,
    binary_name: &str,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    require_active(cancellation)?;
    let file = fs::File::open(archive_path).map_err(|error| error.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|error| error.to_string())?;
    if archive.len() != 1 {
        return Err("Deno archive must contain exactly one executable".to_string());
    }
    let mut entry = archive.by_index(0).map_err(|error| error.to_string())?;
    let enclosed = entry
        .enclosed_name()
        .ok_or("Deno archive contains an unsafe path")?;
    if !entry.is_file() || enclosed.components().count() != 1 || enclosed != Path::new(binary_name)
    {
        return Err("Deno archive contains an unexpected entry".to_string());
    }
    let expected_size = entry.size();
    if expected_size > MAX_DENO_BINARY_BYTES {
        return Err("Deno executable exceeds the extraction limit".to_string());
    }
    let mut output = fs::File::create(binary).map_err(|error| error.to_string())?;
    let mut copied = 0_u64;
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        require_active(cancellation)?;
        let read = entry.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        copied = copied.saturating_add(read as u64);
        if copied > MAX_DENO_BINARY_BYTES {
            return Err("Deno executable exceeds the extraction limit".to_string());
        }
        output
            .write_all(&buffer[..read])
            .map_err(|error| error.to_string())?;
    }
    if copied != expected_size {
        return Err("Deno executable size does not match its archive entry".to_string());
    }
    output.flush().map_err(|error| error.to_string())
}

fn require_active(cancellation: &CancellationToken) -> Result<(), String> {
    if cancellation.is_cancelled() {
        Err(super::super::preparation::RESOURCE_PREPARATION_CANCELLED.to_string())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
